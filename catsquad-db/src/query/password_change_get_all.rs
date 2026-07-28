use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{
    Db, DbPasswordChange, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserPasswordChangeGetAllErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),
}

impl Db {
    pub async fn password_change_get_all(
        &self,
    ) -> Result<Vec<DbPasswordChange>, DbUserPasswordChangeGetAllErr> {
        let query = "SELECT *, user.* FROM password_change ORDER BY created_at DESC;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbUserPasswordChangeGetAllErr::DB(err)
                }
            })
            .and_then_take_all(0)
    }
}

#[tokio::test]
async fn test_password_change_get_all() {
    init_log();

    let db = Db::mem(0).await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    db.user_add(0, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await
        .unwrap();
    let users = db.password_change_add(0, "hey@hey.com", 10).await.unwrap();
    let password_changes = db.password_change_get_all().await.unwrap();
    assert_eq!(password_changes.len(), 1);
}
