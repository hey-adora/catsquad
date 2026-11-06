use crate::{Db, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbConfirmToken {
    #[sqlx(rename = "token_id")]
    #[sqlx(try_from = "XUuid")]
    pub id: Uuid,
    #[sqlx(rename = "token_user_username")]
    pub user_username: String,
    #[sqlx(rename = "token_label")]
    #[sqlx(try_from = "String")]
    pub label: DbConfirmTokenLabel,
    #[sqlx(rename = "token_used")]
    pub used: bool,
    #[sqlx(rename = "token_expires_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub expires_at: u64,
    #[sqlx(rename = "token_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "token_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum DbConfirmTokenLabel {
    PasswordChange,
    AccountDeletion,
}

impl Display for DbConfirmTokenLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            DbConfirmTokenLabel::PasswordChange => "password_change",
            DbConfirmTokenLabel::AccountDeletion => "account_deletion",
        };

        write!(f, "{}", text)
    }
}

impl From<&str> for DbConfirmTokenLabel {
    fn from(value: &str) -> Self {
        match value {
            "password_change" => DbConfirmTokenLabel::PasswordChange,
            "account_deletion" => DbConfirmTokenLabel::AccountDeletion,
            _ => panic!("database has invalid labels for tokens table"),
        }
    }
}

impl From<String> for DbConfirmTokenLabel {
    fn from(value: String) -> Self {
        DbConfirmTokenLabel::from(value.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DbConfirmTokenAddErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user with \"{0}\" username not found")]
    UserNotFound(String),
}

impl Db {
    pub async fn confirm_token_define(&self) {
        let pool = &self.db;
        let query = "
            CREATE TABLE tokens (
                token_id uuid PRIMARY KEY DEFAULT uuidv7(),
                token_user_username varchar NOT NULL references users(user_username) ON UPDATE CASCADE ON DELETE CASCADE,
                token_label varchar NOT NULL,
                token_used bool DEFAULT FALSE,
                token_expires_at timestamp NOT NULL,
                token_modified_at timestamp NOT NULL,
                token_created_at timestamp NOT NULL
            );
        ";
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
    }

    pub async fn confirm_token_add(
        &self,
        time: u64,
        username: impl Into<String>,
        expires: u64,
        label: DbConfirmTokenLabel,
    ) -> Result<DbConfirmToken, DbConfirmTokenAddErr> {
        let pool = &self.db;
        let username = username.into();

        let query = "
            INSERT INTO tokens (
                    token_user_username,
                    token_expires_at,
                    token_modified_at,
                    token_created_at,
                    token_label
                )
                VALUES ( $1, $2, $3, $3, $4 )
                RETURNING token_id
        ";

        debug!("about to run {query}");

        let result = sqlx::query_as(query)
            .bind(&username)
            .bind(XTimestamp(expires as i64))
            .bind(XTimestamp(time as i64))
            .bind(label.to_string())
            .fetch_one(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::Database(err))
                if err.is_foreign_key_violation()
                    && err.constraint() == Some("tokens_token_user_username_fkey") =>
            {
                return Err(DbConfirmTokenAddErr::UserNotFound(username));
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbConfirmTokenAddErr::Db(err));
            }
        };

        let (XUuid(token),): (XUuid,) = result;

        let token = DbConfirmToken {
            id: token,
            user_username: username,
            label,
            used: false,
            expires_at: expires,
            modified_at: time,
            created_at: time,
        };

        Ok(token)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_confirm_token_add() {
    init_log();

    let db = Db::test_db(0, "test_token_add").await;

    let username = "hey";
    let email = "hey@heyadora.com";
    let invite1 = db.invite_add(0, email, 1).await.unwrap();
    assert_eq!(invite1.email, email);

    let _user = db
        .user_add(0, username, "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let _result = db
        .confirm_token_add(0, username, 10, DbConfirmTokenLabel::PasswordChange)
        .await
        .unwrap();

    let result = db
        .confirm_token_add(0, "invalid", 10, DbConfirmTokenLabel::PasswordChange)
        .await;
    // matches!()
    assert!(matches!(result, Err(DbConfirmTokenAddErr::UserNotFound(_))));
}
