use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserGetByEmailErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_get_by_email(
        &self,
        email: impl Into<String>,
    ) -> Result<DbUser, DbUserGetByEmailErr> {
        let query = "SELECT * FROM user WHERE email = $email;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("email", email.into()))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbUserGetByEmailErr::DB(err)
                }
            })
            .and_then_take_or(0, DbUserGetByEmailErr::NotFound)
    }
}

#[tokio::test]
async fn test_user_get_by_email() {
    init_log();

    let db = Db::mem().await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    db.user_add(0, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await
        .unwrap();
    let user = db.user_get_by_email("hey@hey.com").await.unwrap();
    assert_eq!(user.email, "hey@hey.com");
}
