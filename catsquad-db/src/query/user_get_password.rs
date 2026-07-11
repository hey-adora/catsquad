use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserGetPasswordErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_get_password(
        &self,
        email: impl Into<String>,
    ) -> Result<String, DbUserGetPasswordErr> {
        let query = "(SELECT password FROM user WHERE email = $email).password";

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("email", email.into()))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbUserGetPasswordErr::DB(err)
                }
            })
            .and_then_take_or(0, DbUserGetPasswordErr::NotFound)
    }
}

#[tokio::test]
async fn test_user_get_password() {
    init_log();

    let db = Db::mem().await;

    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    db.user_add(0, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await
        .unwrap();
    let password = db.user_get_password("hey@hey.com").await.unwrap();
    assert_eq!(password, "hey");
}
