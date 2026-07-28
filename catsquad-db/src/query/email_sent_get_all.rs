use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbEmailSent, DbInvite, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbEmailSentGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn email_sent_get_all(&self) -> Result<Vec<DbEmailSent>, DbEmailSentGetAllErr> {
        let query = "SELECT * FROM email_sent ORDER BY created_at DESC;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbEmailSentGetAllErr::Db(err)
                }
            })
            .and_then_take_all(0)
    }
}

#[tokio::test]
async fn test_email_sent_get_all() {
    init_log();

    let db = Db::mem(0).await;
    let invite = db
        .email_sent_add(
            0,
            crate::DbEmailSentReason::InviteAdd,
            "hey@heyadora.com",
            "hello",
        )
        .await
        .unwrap();
    let invites = db.email_sent_get_all().await.unwrap();
    assert_eq!(invites.len(), 1);
}
