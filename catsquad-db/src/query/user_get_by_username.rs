use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserGetByUsernameErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_get_by_username(
        &self,
        username: impl Into<String>,
    ) -> Result<DbUser, DbUserGetByUsernameErr> {
        let query = "SELECT * FROM user WHERE username = $username;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("username", username.into()))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbUserGetByUsernameErr::DB(err)
                }
            })
            .and_then_take_or(0, DbUserGetByUsernameErr::NotFound)
    }
}

#[tokio::test]
async fn test_user_get_by_username() {
    init_log();

    let db = Db::mem(0).await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let invite2 = db.invite_add(0, "hey2@hey.com", 10).await.unwrap();

    db.user_add(0, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await
        .unwrap();
    db.user_add(0, "hey2", "hey", invite2.id.key.clone(), 10, 10)
        .await
        .unwrap();
    let user = db.user_get_by_username("hey2").await.unwrap();
    assert_eq!(user.username, "hey2");
    let user = db.user_get_by_username("hey3").await;
    assert_eq!(user, Err(DbUserGetByUsernameErr::NotFound));
}
