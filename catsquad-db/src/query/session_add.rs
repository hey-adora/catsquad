use crate::{Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};
use catsquad_log::prelude::*;
use std::fmt::Display;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbSession {
    pub id: RecordId,
    pub user: DbUser,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbSessionAddErr {
    #[error("user {0} not found")]
    UserNotFound(String),

    #[error("db error {0}")]
    Db(#[from] surrealdb::Error),
}

pub fn create_session_id(id: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("session", id)
}

impl Db {
    pub async fn session_define(&self) {
        let query = "
                DEFINE TABLE session SCHEMAFULL;
                DEFINE FIELD user ON TABLE session TYPE record<user>;
                DEFINE FIELD modified_at ON TABLE session TYPE number;
                DEFINE FIELD created_at ON TABLE session TYPE number;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn session_add(
        &self,
        time: u128,
        email: impl Into<String>,
    ) -> Result<DbSession, DbSessionAddErr> {
        let email = email.into();
        let query = r#"
                 BEGIN TRANSACTION;
                 LET $user = SELECT id FROM ONLY user WHERE email = $email;
                 CREATE session SET user = $user.id, modified_at = $time, created_at = $time RETURN *, user.*;
                 COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("email", email.clone()))
            .await
            .check_better(|err| match err {
                err if err.field_value_null("user") => DbSessionAddErr::UserNotFound(email),
                err => {
                    error!("unexpected db error {err}");
                    err.into()
                }
            })
            .and_then_take_expect(2)
    }
}

#[tokio::test]
async fn test_session_add() {
    init_log();

    let db = Db::mem(0).await;

    let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    let result = db
        .user_add(0, "hey", "hey", invite.id.key, 10, 10)
        .await
        .unwrap();

    let session = db.session_add(0, "hey@hey.com").await.unwrap();
    assert_eq!(session.user.username, "hey");

    let result = db.session_add(0, "hey2@hey.com").await;
    assert!(matches!(result, Err(DbSessionAddErr::UserNotFound(_))));
}
