use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserGetAllErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),
}

impl Db {
    pub async fn user_get_all(&self) -> Result<Vec<DbUser>, DbUserGetAllErr> {
        let query = "SELECT * FROM user ORDER BY created_at DESC;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbUserGetAllErr::DB(err)
                }
            })
            .and_then_take_all(0)
    }
}

#[tokio::test]
async fn test_user_get_all() {
    init_log();

    let db = Db::mem().await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    db.user_add(0, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await
        .unwrap();
    let users = db.user_get_all().await.unwrap();
    assert_eq!(users.len(), 1);
}
