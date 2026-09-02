use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbEmailSent {
    #[sqlx(rename = "email_sent_id")]
    pub id: i64,
    #[sqlx(rename = "email_sent_body")]
    pub body: String,
    #[sqlx(rename = "email_sent_to_email")]
    pub to_email: String,
    #[sqlx(rename = "email_sent_reason")]
    pub reason: String,
    #[sqlx(rename = "email_sent_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "email_sent_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbEmailSentReason {
    InviteAdd,
    SessionAdd,
    UserEmailChangeAddCurrent,
    UserEmailChangeAddNew,
    UserEmailChangeFinish,
    // UserEmailChangeConfirmCurrent,
    // UserEmailChangeConfirmNew,
    UserUsernameChange,
    UserPasswordChangeAdd,
    UserPasswordChangeConfirm,
    UserPasswordResetAdd,
    UserPasswordResetConfirm,
}

impl Display for DbEmailSentReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            DbEmailSentReason::InviteAdd => "invite_add",
            DbEmailSentReason::SessionAdd => "session_add",
            DbEmailSentReason::UserUsernameChange => "user_username_change",
            DbEmailSentReason::UserEmailChangeAddCurrent => "user_email_change_add_current",
            DbEmailSentReason::UserEmailChangeAddNew => "user_email_change_add_new",
            DbEmailSentReason::UserEmailChangeFinish => "user_email_change_finish",
            // DbEmailSentReason::UserEmailChangeConfirmCurrent => "user_email_change_confirm_current",
            // DbEmailSentReason::UserEmailChangeConfirmNew => "user_email_change_confirm_new",
            DbEmailSentReason::UserPasswordChangeAdd => "user_password_change_add",
            DbEmailSentReason::UserPasswordChangeConfirm => "user_password_change_confirm",
            DbEmailSentReason::UserPasswordResetAdd => "user_password_reset_add",
            DbEmailSentReason::UserPasswordResetConfirm => "user_password_reset_confirm",
        };

        write!(f, "{}", text)
    }
}

// impl From<String>

#[derive(Debug, thiserror::Error)]
pub enum DbEmailSentAddErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_sent_define(&self) {
        let pool = &self.db;
        let query = "
            CREATE TABLE emails_sent (
                email_sent_id int8 PRIMARY KEY generated always as identity,
                email_sent_body text NOT NULL,
                email_sent_to_email varchar NOT NULL,
                email_sent_reason varchar NOT NULL,
                email_sent_modified_at timestamp NOT NULL,
                email_sent_created_at timestamp NOT NULL
            );
        ";
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
    }

    pub async fn email_sent_add(
        &self,
        time: u64,
        reason: DbEmailSentReason,
        to_email: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<DbEmailSent, DbEmailSentAddErr> {
        let pool = &self.db;
        let to_email = to_email.into();
        let body = body.into();
        let reason = reason.to_string();

        let query = "
            INSERT INTO emails_sent (
                    email_sent_body,
                    email_sent_to_email,
                    email_sent_reason,
                    email_sent_modified_at,
                    email_sent_created_at
                )
                VALUES ($1, $2, $3, $4, $4)
                RETURNING email_sent_id
        ";

        let result = sqlx::query_as(query)
            .bind(&body)
            .bind(&to_email)
            .bind(&reason) // TODO make it &'static str
            .bind(XTimestamp(time as i64))
            .fetch_one(pool)
            .await;

        debug!("query {query} result {result:?}");

        let (emails_sent_id,): (i64,) = match result {
            Ok(v) => v,
            // Err(sqlx::Error::RowNotFound) => {
            //     return Err(DbEmailSentAddErr::UserNotFound);
            // }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbEmailSentAddErr::Db(err));
            }
        };

        let email_sent = DbEmailSent {
            id: emails_sent_id,
            body,
            to_email,
            reason,
            modified_at: time,
            created_at: time,
        };

        Ok(email_sent)

    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_sent_add() {
    init_log();

    let db = Db::test_db(0, "test_email_sent_add").await;

    let email = db
        .email_sent_add(0, DbEmailSentReason::InviteAdd, "prime@heyadora.com", "wtf")
        .await
        .unwrap();

    assert_eq!(email.body, "wtf");
    assert_eq!(email.reason, DbEmailSentReason::InviteAdd.to_string());
    assert_eq!(email.to_email, "prime@heyadora.com");
}
