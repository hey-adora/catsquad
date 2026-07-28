use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserUpdatePasswordByEmailErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_update_password_by_email(
        &self,
        time: u128,
        email: impl Into<String>,
        new_password: impl Into<String>,
    ) -> Result<DbUser, DbUserUpdatePasswordByEmailErr> {
        let query =
            "UPDATE user SET modified_at = $time, password = $new_password WHERE email = $email;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("email", email.into()))
            .bind(("new_password", new_password.into()))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbUserUpdatePasswordByEmailErr::DB(err)
                }
            })
            .and_then_take_or(0, DbUserUpdatePasswordByEmailErr::NotFound)
    }
}

#[tokio::test]
async fn test_user_update_password_by_email() {
    init_log();

    let db = Db::mem(0).await;

    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await
        .unwrap();
    assert_eq!(user.password, "hey");
    let user = db
        .user_update_password_by_email(0, "hey@hey.com", "hey2")
        .await
        .unwrap();
    assert_eq!(user.password, "hey2");
}
