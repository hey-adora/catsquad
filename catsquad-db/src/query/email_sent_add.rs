use std::fmt::Display;

use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbEmailSent {
    pub id: RecordId,
    pub body: String,
    pub to_email: String,
    pub reason: String,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub enum DbEmailSentReason {
    ConfirmInvite,
    ConfirmPasswordChange,
    ConfirmEmailChange,
    ConfirmEmailChangeNewEmail,
}

impl Display for DbEmailSentReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            DbEmailSentReason::ConfirmInvite => "confirm_invite",
            DbEmailSentReason::ConfirmPasswordChange => "confirm_password_change",
            DbEmailSentReason::ConfirmEmailChange => "confirm_email_change",
            DbEmailSentReason::ConfirmEmailChangeNewEmail => "confirm_email_change_new_email",
        };

        write!(f, "{}", text)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbEmailSentAddErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

pub fn create_email_sent_id(id: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("email_sent", id)
}

impl Db {
    pub async fn email_sent_define(&self) {
        let query = "
                DEFINE TABLE email_sent SCHEMAFULL;
                DEFINE FIELD reason ON TABLE email_sent TYPE string;
                DEFINE FIELD to_email ON TABLE email_sent TYPE string;
                DEFINE FIELD body ON TABLE email_sent TYPE string;
                DEFINE FIELD modified_at ON TABLE email_sent TYPE number;
                DEFINE FIELD created_at ON TABLE email_sent TYPE number;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn email_sent_add(
        &self,
        time: u128,
        reason: DbEmailSentReason,
        to_email: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<DbEmailSent, DbEmailSentAddErr> {
        let query = r#"
                 CREATE email_sent SET
                    reason = $reason,
                    to_email = $to_email,
                    body = $body,
                    modified_at = $time,
                    created_at = $time;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("reason", reason.to_string()))
            .bind(("to_email", to_email.into()))
            .bind(("body", body.into()))
            .await
            .check_better(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbEmailSentAddErr::Db(err)
                }
            })
            .and_then_take_expect(0)
    }
}

#[tokio::test]
async fn test_email_sent_add() {
    init_log();

    let db = Db::mem().await;

    let email = db
        .email_sent_add(
            0,
            DbEmailSentReason::ConfirmInvite,
            "prime@heyadora.com",
            "wtf",
        )
        .await
        .unwrap();

    assert_eq!(email.body, "wtf");
    assert_eq!(email.reason, DbEmailSentReason::ConfirmInvite.to_string());
    assert_eq!(email.to_email, "prime@heyadora.com");
}
