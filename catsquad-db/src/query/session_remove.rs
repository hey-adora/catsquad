use crate::{
    Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    query::session_add::create_session_id,
};
use catsquad_log::prelude::*;
use std::fmt::Display;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbSessionRemoveErr {
    #[error("db error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn session_remove(
        &self,
        session_key: impl Into<RecordIdKey>,
    ) -> Result<(), DbSessionRemoveErr> {
        let session_id = create_session_id(session_key);
        let query = r#"
                 DELETE $session_id;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("session_id", session_id))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbSessionRemoveErr::Db(err)
                }
            })
            .map(|_| ())
    }
}

#[tokio::test]
async fn test_session_remove() {
    init_log();

    let db = Db::mem(0).await;

    let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    let result = db
        .user_add(0, "hey", "hey", invite.id.key, 10, 10)
        .await
        .unwrap();
    let session = db.session_add(0, "hey@hey.com").await.unwrap();

    db.session_remove("invalid").await.unwrap();

    let result = db.session_get_by_key(session.id.key.clone()).await;
    assert!(result.is_ok());

    db.session_remove(session.id.key.clone()).await.unwrap();

    let result = db.session_get_by_key(session.id.key.clone()).await;
    assert!(result.is_err());
}
