use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbInvite, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbInviteGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn invite_get_all(&self) -> Result<Vec<DbInvite>, DbInviteGetAllErr> {
        let query = "SELECT * FROM invite ORDER BY created_at DESC;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbInviteGetAllErr::Db(err)
                }
            })
            .and_then_take_all(0)
    }
}

#[tokio::test]
async fn test_invite_get_all() {
    init_log();

    let db = Db::mem().await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let invites = db.invite_get_all().await.unwrap();
    assert_eq!(invites.len(), 1);
}
