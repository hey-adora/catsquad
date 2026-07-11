use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserUpdatePasswordByIdErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_update_password_by_id(
        &self,
        time: u128,
        user: RecordId,
        new_password: impl Into<String>,
    ) -> Result<DbUser, DbUserUpdatePasswordByIdErr> {
        let query =
            "UPDATE user SET modified_at = $time, password = $new_password WHERE id = $user_id;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user))
            .bind(("new_password", new_password.into()))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbUserUpdatePasswordByIdErr::DB(err)
                }
            })
            .and_then_take_or(0, DbUserUpdatePasswordByIdErr::NotFound)
    }
}

#[tokio::test]
async fn test_user_update_password_by_id() {
    init_log();

    let db = Db::mem().await;

    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await
        .unwrap();
    assert_eq!(user.password, "hey");
    let user = db
        .user_update_password_by_id(0, user.id.clone(), "hey2")
        .await
        .unwrap();
    assert_eq!(user.password, "hey2");
}
