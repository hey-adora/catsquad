use crate::{Db, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;
#[cfg(test)]
use catsquad_shared::SessionTokenKind;
use catsquad_shared::UserState;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbUser {
    #[sqlx(rename = "user_id")]
    pub id: i64,
    #[sqlx(rename = "user_username")]
    pub username: String,
    #[sqlx(rename = "user_email")]
    pub email: String,
    #[sqlx(rename = "user_password")]
    pub password: String,
    #[sqlx(rename = "user_support")]
    pub support: String,
    #[sqlx(rename = "user_aboutme")]
    pub aboutme: String,
    #[sqlx(rename = "user_state")]
    pub state: String,
    #[sqlx(rename = "user_roles")]
    pub roles: Vec<i64>,
    #[sqlx(rename = "user_pfp_image_hash")]
    pub pfp_image_hash: i64,
    #[sqlx(default, rename = "user_pfp_image_proccesed")]
    pub pfp_image_proccesed: bool,
    #[sqlx(rename = "user_likes_count")]
    #[sqlx(try_from = "i64")]
    pub likes_count: u32, // likes that user receives
    #[sqlx(rename = "user_followers_count")]
    #[sqlx(try_from = "i64")]
    pub followers_count: u32,
    #[sqlx(rename = "user_following_count")]
    #[sqlx(try_from = "i64")]
    pub following_count: u32,
    #[sqlx(rename = "user_used_storage_bytes")]
    #[sqlx(try_from = "i64")]
    pub used_storage_bytes: u32,
    #[sqlx(rename = "user_max_storage_per_file_bytes")]
    #[sqlx(try_from = "i64")]
    pub max_storage_per_image_bytes: u32,
    #[sqlx(rename = "user_max_storage_bytes")]
    #[sqlx(try_from = "i64")]
    pub max_storage_bytes: u32,
    #[sqlx(rename = "user_state_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub state_modified_at: u64,
    #[sqlx(rename = "user_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "user_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

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
                user_id int8 PRIMARY KEY generated always as identity,
                user_username varchar NOT NULL,
                user_email varchar NOT NULL,
                user_password varchar NOT NULL,
                user_support text DEFAULT '',
                user_aboutme text DEFAULT '',
                user_state varchar NOT NULL,
                user_roles int8[] DEFAULT array[]::int8[],
                user_pfp_image_hash int8 DEFAULT 0,
                user_likes_count int8 DEFAULT 0,
                user_followers_count int8 DEFAULT 0,
                user_following_count int8 DEFAULT 0,
                user_used_storage_bytes int8 DEFAULT 0,
                user_max_storage_per_file_bytes int8 NOT NULL,
                user_max_storage_bytes int8 NOT NULL,
                user_state_modified_at timestamp NOT NULL,
                user_modified_at timestamp NOT NULL,
                user_created_at timestamp NOT NULL
            );
            CREATE UNIQUE INDEX user_username_idx ON users (user_username);
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
        let username = username.into();
        let password = password.into();

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
        let user_id = {
            let query = "
            INSERT INTO users (
                    user_username,
                    user_email,
                    user_password,
                    user_max_storage_bytes,
                    user_max_storage_per_file_bytes,
                    user_state_modified_at,
                    user_modified_at,
                    user_created_at,
                    user_state
                )
                VALUES ( $1, $2, $3, $4, $5, $6, $6, $6, $7 )
                RETURNING user_id
            ";

            let result = sqlx::query_as(query)
                .bind(&username)
                .bind(&email)
                .bind(&password)
                .bind(max_storage_bytes as i64)
                .bind(max_storage_per_file_bytes as i64)
                .bind(XTimestamp(time as i64))
                .bind(UserState::Active.as_str())
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (user_id,): (i64,) = match result {
                Ok(v) => v,
                Err(sqlx::Error::Database(err))
                    if err.is_unique_violation() && err.constraint() == Some("user_email_idx") =>
                {
                    return Err(DbUserAddErr::EmailIsTaken);
                }
                Err(sqlx::Error::Database(err))
                    if err.is_unique_violation()
                        && err.constraint() == Some("user_username_idx") =>
                {
                    return Err(DbUserAddErr::UsernameIsTaken);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserAddErr::Db(err));
                }
            };

            user_id
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
            id: user_id,
            username,
            email,
            password,
            support: String::new(),
            aboutme: String::new(),
            state: UserState::Active.to_string(),
            roles: Vec::new(),
            pfp_image_hash: 0,
            pfp_image_proccesed: false,
            likes_count: 0,
            followers_count: 0,
            following_count: 0,
            used_storage_bytes: 0,
            max_storage_bytes,
            max_storage_per_image_bytes: max_storage_per_file_bytes,
            state_modified_at: time,
            modified_at: time,
            created_at: time,
        })
    }
}

#[cfg(test)]
pub async fn user_add_with_roles<TValue>(
    db: &Db,
    time: u64,
    max_storage_bytes: u32,
    max_storage_per_file_bytes: u32,
    username: impl Into<String>,
    password: impl Into<String>,
    email: impl Into<String>,
    permissions: Vec<TValue>,
) -> (DbUser, SessionTokenKind)
where
    TValue: Into<String>,
{
    use catsquad_shared::SessionTokenKind;

    let username = username.into();
    let password = password.into();
    let email = email.into();
    let role_name = format!("{}_role", username);

    let (user1, user1_session) = {
        let invite1 = db.invite_add(time, &email, time).await.unwrap();

        let user1 = db
            .user_add(
                time,
                username,
                password,
                invite1.token.clone(),
                max_storage_bytes,
                max_storage_per_file_bytes,
            )
            .await
            .unwrap();

        let user1_session = db.session_add(time, email).await.unwrap();

        (user1, SessionTokenKind::User(user1_session.token))
    };

    db.role_add(time, SessionTokenKind::Root, &role_name, permissions)
        .await
        .unwrap();

    db.user_update_role_add(time, user1.username.clone(), role_name)
        .await
        .unwrap();

    (user1, user1_session)
}

#[cfg(test)]
#[tokio::test]
async fn test_user_add() {
    init_log();
    let db = Db::test_db(0, "test_user_add").await;

    // assert errors
    {
        let result = db
            .user_add(0, "hey", "hey", 0_u128.to_be_bytes(), 10, 10)
            .await;
        assert!(matches!(result, Err(DbUserAddErr::InviteNotFound)));

        let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
        let result = db.user_add(2, "hey", "hey", invite.token, 10, 10).await;
        assert!(matches!(result, Err(DbUserAddErr::InviteExpired)));
    }

    // assert success
    let invite = {
        let invite = db.invite_add(2, "hey@hey.com", 3).await.unwrap();
        let result = db
            .user_add(3, "hey", "hey", invite.token.clone(), 10, 5)
            .await
            .unwrap();
        assert_eq!(result.username, "hey");
        assert_eq!(result.email, "hey@hey.com");
        assert_eq!(result.password, "hey");
        assert_eq!(result.followers_count, 0);
        assert_eq!(result.following_count, 0);
        assert_eq!(result.likes_count, 0);
        assert_eq!(result.used_storage_bytes, 0);
        assert_eq!(result.max_storage_bytes, 10);
        assert_eq!(result.max_storage_per_image_bytes, 5);
        assert_eq!(result.modified_at, 3);
        assert_eq!(result.created_at, 3);

        invite
    };

    // assert errors
    {
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

    // assert error email is taken
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
