use crate::{Db, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbEmailChange {
    #[sqlx(rename = "email_change_id")]
    pub id: i64,
    #[sqlx(rename = "email_change_user_username")]
    pub user_username: String,
    #[sqlx(rename = "email_change_current_email")]
    pub current_email: String,
    #[sqlx(rename = "email_change_current_token")]
    #[sqlx(try_from = "XUuid")]
    pub current_token: Uuid,
    #[sqlx(rename = "email_change_current_used")]
    pub current_used: bool,
    #[sqlx(rename = "email_change_new_email")]
    pub new_email: String,
    #[sqlx(rename = "email_change_new_token")]
    #[sqlx(try_from = "XUuid")]
    pub new_token: Uuid,
    #[sqlx(rename = "email_change_new_used")]
    pub new_used: bool,
    #[sqlx(rename = "email_change_completed")]
    pub completed: bool,
    #[sqlx(rename = "email_change_expires_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub expires_at: u64,
    #[sqlx(rename = "email_change_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "email_change_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbEmailChangeAddErr {
    #[error("user not found")]
    UserNotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_change_define(&self) {
        let pool = &self.db;
        // add foreign key FILE
        let query = "
            CREATE TABLE emails_changes (
                email_change_id int8 PRIMARY KEY generated always as identity,
                email_change_user_username varchar NOT NULL references users(user_username) ON UPDATE CASCADE ON DELETE CASCADE,
                email_change_current_email varchar NOT NULL,
                email_change_current_token uuid DEFAULT uuidv7(),
                email_change_current_used bool DEFAULT FALSE,
                email_change_new_email varchar DEFAULT '',
                email_change_new_token uuid DEFAULT uuidv7(),
                email_change_new_used bool DEFAULT FALSE,
                email_change_completed bool DEFAULT FALSE,
                email_change_expires_at timestamp NOT NULL,
                email_change_modified_at timestamp NOT NULL,
                email_change_created_at timestamp NOT NULL
            );
            CREATE INDEX email_change_user_username_idx ON emails_changes (email_change_user_username);
        ";
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
    }

    pub async fn email_change_add(
        &self,
        time: u64,
        user_username: impl Into<String>,
        expires: u64,
    ) -> Result<DbEmailChange, DbEmailChangeAddErr> {
        let pool = &self.db;
        let user_username = user_username.into();

        let query = "
            INSERT INTO emails_changes (
                    email_change_user_username,
                    email_change_current_email,
                    email_change_expires_at,
                    email_change_modified_at,
                    email_change_created_at
                )
                SELECT $1, user_email, $2, $3, $3 FROM users WHERE user_username = $1
                RETURNING email_change_id, email_change_current_token, email_change_new_token, email_change_current_email
        ";
        debug!("about to run {query}");
        let result = sqlx::query_as(query)
            .bind(&user_username)
            .bind(XTimestamp(expires as i64))
            .bind(XTimestamp(time as i64))
            .fetch_one(pool)
            .await;

        let (
            email_change_id,
            XUuid(email_change_current_token),
            XUuid(email_change_new_token),
            email_change_current_email,
        ): (i64, XUuid, XUuid, String) = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbEmailChangeAddErr::UserNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbEmailChangeAddErr::Db(err));
            }
        };

        let email_change = DbEmailChange {
            id: email_change_id,
            user_username,
            current_email: email_change_current_email,
            current_token: email_change_current_token,
            current_used: false,
            new_email: String::new(),
            new_token: email_change_new_token,
            new_used: false,
            completed: false,
            expires_at: expires,
            modified_at: time,
            created_at: time,
        };

        Ok(email_change)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_add() {
    init_log();

    let db = Db::test_db(0, "test_email_change_add").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let _result = db
        .email_change_add(0, user.username.clone(), 10)
        .await
        .unwrap();

    let result = db.email_change_add(0, "invalid", 10).await;
    assert!(matches!(result, Err(DbEmailChangeAddErr::UserNotFound)));
}
