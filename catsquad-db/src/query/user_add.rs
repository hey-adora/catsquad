use crate::{Db, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbUser {
    #[sqlx(rename = "user_username")]
    pub username: String,
    #[sqlx(rename = "user_email")]
    pub email: String,
    #[sqlx(rename = "user_password")]
    pub password: String,
    #[sqlx(rename = "user_used_storage_bytes")]
    #[sqlx(try_from = "i64")]
    pub used_storage_bytes: u32,
    #[sqlx(rename = "user_max_storage_per_file_bytes")]
    #[sqlx(try_from = "i64")]
    pub max_storage_per_file_bytes: u32,
    #[sqlx(rename = "user_max_storage_bytes")]
    #[sqlx(try_from = "i64")]
    pub max_storage_bytes: u32,
    #[sqlx(rename = "user_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "user_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

// #[derive(Debug, Clone, sqlx::FromRow)]
// pub struct DbUserRaw {
//     pub username: String,
//     pub email: String,
//     pub password: String,
//     pub used_storage_bytes: i64,
//     pub max_storage_per_file_bytes: i64,
//     pub max_storage_bytes: i64,
//     pub modified_at: TimeStamp,
//     pub created_at: TimeStamp,
// }

#[derive(Debug, thiserror::Error)]
pub enum DbUserAddErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("email is taken")]
    EmailIsTaken,

    #[error("username is taken")]
    UsernameIsTaken,

    #[error("invite not found")]
    InviteNotFound,

    #[error("invite already used")]
    InviteAlreadyUsed,

    #[error("invite expired")]
    InviteExpired,
}

impl Db {
    pub async fn user_define(&self) {
        let pool = &self.db;
        let query = "
            CREATE TABLE users (
                user_username varchar(32) PRIMARY KEY,
                user_email varchar(100) NOT NULL,
                user_password varchar(255) NOT NULL,
                user_used_storage_bytes int8 DEFAULT 0,
                user_max_storage_per_file_bytes int8 NOT NULL,
                user_max_storage_bytes int8 NOT NULL,
                user_modified_at timestamp NOT NULL,
                user_created_at timestamp NOT NULL
            );
            CREATE UNIQUE INDEX user_email_idx ON users (user_email);
        ";
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
    }

    pub async fn user_add(
        &self,
        time: u64,
        username: impl Into<String>,
        password: impl Into<String>,
        invite_token: Uuid,
        max_storage_bytes: u32,
        max_storage_per_file_bytes: u32,
    ) -> Result<DbUser, DbUserAddErr> {
        let pool = &self.db;

        let mut tx = pool.begin().await?;

        // get invite
        let email = {
            let query = "SELECT invite_email, invite_used, invite_expires_at FROM invites WHERE invite_token=$1";

            debug!("about to run {query}");

            let result = sqlx::query_as(query)
                .bind(XUuid(invite_token))
                .fetch_one(&mut *tx)
                .await;

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbUserAddErr::InviteNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserAddErr::Db(err));
                }
            };

            let (email, is_used, XTimestamp(expires_at)): (String, bool, XTimestamp) = result;
            let expires_at = expires_at as u64;

            debug!(
                "query: {query}\nresult: email:{email} is_used: {is_used} expires_at: {expires_at}"
            );

            if is_used {
                return Err(DbUserAddErr::InviteAlreadyUsed);
            }

            if expires_at < time {
                return Err(DbUserAddErr::InviteExpired);
            }

            email
        };

        // add user
        let (username, password) = {
            let username = username.into();
            let password = password.into();

            let query = "
            INSERT INTO users (
                    user_username,
                    user_email,
                    user_password,
                    user_max_storage_bytes,
                    user_max_storage_per_file_bytes,
                    user_modified_at,
                    user_created_at
                )
                VALUES ( $1, $2, $3, $4, $5, $6, $6 )
        ";
            debug!("about to run {query}");
            let result = sqlx::query(query)
                .bind(&username)
                .bind(&email)
                .bind(&password)
                .bind(max_storage_bytes as i64)
                .bind(max_storage_per_file_bytes as i64)
                .bind(XTimestamp(time as i64))
                .execute(&mut *tx)
                .await;

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::Database(err))
                    if err.is_unique_violation() && err.constraint() == Some("user_email_idx") =>
                {
                    return Err(DbUserAddErr::EmailIsTaken);
                }
                Err(sqlx::Error::Database(err))
                    if err.is_unique_violation() && err.constraint() == Some("users_pkey") =>
                {
                    return Err(DbUserAddErr::UsernameIsTaken);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserAddErr::Db(err));
                }
            };
            debug!("query: {query}\nresult: {result:#?}");

            (username, password)
        };

        // update invite
        {
            let query = "UPDATE invites SET invite_used = TRUE, invite_modified_at = $1 WHERE invite_token = $2";

            debug!("about to run {query}");

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(XUuid(invite_token))
                .execute(&mut *tx)
                .await?;

            debug!("query: {query}\nresult: {result:#?}");
        }

        tx.commit().await?;

        Ok(DbUser {
            username,
            email,
            password,
            used_storage_bytes: 0,
            max_storage_bytes,
            max_storage_per_file_bytes,
            modified_at: time,
            created_at: time,
        })
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_add() {
    init_log();
    let db = Db::test_db(0, "test_user_add").await;

    // simple errors
    {
        let result = db
            .user_add(0, "hey", "hey", 0_u128.to_be_bytes(), 10, 10)
            .await;
        assert!(matches!(result, Err(DbUserAddErr::InviteNotFound)));

        let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
        let result = db.user_add(2, "hey", "hey", invite.token, 10, 10).await;
        assert!(matches!(result, Err(DbUserAddErr::InviteExpired)));

        // success
        let invite = db.invite_add(2, "hey@hey.com", 3).await.unwrap();
        let result = db
            .user_add(3, "hey", "hey", invite.token.clone(), 10, 5)
            .await
            .unwrap();
        assert_eq!(result.username, "hey");
        assert_eq!(result.email, "hey@hey.com");
        assert_eq!(result.password, "hey");
        assert_eq!(result.used_storage_bytes, 0);
        assert_eq!(result.max_storage_bytes, 10);
        assert_eq!(result.max_storage_per_file_bytes, 5);
        assert_eq!(result.modified_at, 3);
        assert_eq!(result.created_at, 3);

        let result = db
            .user_add(2, "hey", "hey", invite.token.clone(), 10, 10)
            .await;
        assert!(matches!(result, Err(DbUserAddErr::InviteAlreadyUsed)));

        let invite = db.invite_add(2, "hey2@hey.com", 3).await.unwrap();
        let result = db
            .user_add(2, "hey", "hey", invite.token.clone(), 10, 10)
            .await;
        assert!(matches!(result, Err(DbUserAddErr::UsernameIsTaken)));
    }

    // complicated error email is taken
    {
        let invite1 = db.invite_add(2, "hey3@hey.com", 3).await.unwrap();
        let invite2 = db.invite_add(2, "hey3@hey.com", 3).await.unwrap();

        let result = db
            .user_add(2, "hey3", "hey", invite1.token.clone(), 10, 10)
            .await
            .unwrap();

        let result = db
            .user_add(2, "hey4", "hey", invite2.token.clone(), 10, 10)
            .await;

        assert!(matches!(result, Err(DbUserAddErr::EmailIsTaken)));
    }
}
