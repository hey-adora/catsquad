use crate::{
    Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    query::session_add::{DbSession, create_session_id},
};
use catsquad_log::prelude::*;
use std::fmt::Display;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbSessionGetByKeyErr {
    #[error("session {0:?} not found")]
    NotFound(RecordIdKey),

    #[error("db error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    // pub async fn get_session<S: Into<String>>(&self, token: S) -> Result<DBSession, DB404Err> {
    //       let token = token.into();
    //       let session_id = create_session_id(token.clone());
    //       self.db
    //           .query("SELECT *, user.* FROM $session_id;")
    //           .bind(("session_id", session_id))
    //           .await
    //           .check_good(DB404Err::from)
    //           .and_then_take_or(0, DB404Err::NotFound)
    //   }

    pub async fn session_get_by_key(
        &self,
        session_key: impl Into<RecordIdKey>,
    ) -> Result<DbSession, DbSessionGetByKeyErr> {
        let session_key = session_key.into();
        let session_id = create_session_id(session_key.clone());
        let query = r#"
                    SELECT *, user.* FROM $session_id;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("session_id", session_id))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbSessionGetByKeyErr::Db(err)
                }
            })
            .and_then_take_or(0, DbSessionGetByKeyErr::NotFound(session_key))
    }
}

#[tokio::test]
async fn test_session_get_by_key() {
    init_log();

    let db = Db::mem(0).await;

    let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    let result = db
        .user_add(0, "hey", "hey", invite.id.key, 10, 10)
        .await
        .unwrap();

    let session = db.session_add(0, "hey@hey.com").await.unwrap();
    assert_eq!(session.user.username, "hey");

    let _result = db.session_get_by_key(session.id.key.clone()).await.unwrap();
    let result = db.session_get_by_key("invalid").await;
    assert!(matches!(result, Err(DbSessionGetByKeyErr::NotFound(_))));
}
