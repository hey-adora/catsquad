use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{
    Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_user_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserUpdateUsernameErr {
    #[error("username already used")]
    UsernameAlreadyUsed,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn user_update_username(
        &self,
        time: u128,
        user_id: RecordId,
        new_username: impl Into<String>,
    ) -> Result<String, DbUserUpdateUsernameErr> {
        let new_username = new_username.into();

        let query = r#"
                 (UPDATE $user_id SET username = $new_username RETURN username).username;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("new_username", new_username))
            .await
            .check_better(|err| match err {
                err if err.index_exists("idx_user_username") => {
                    DbUserUpdateUsernameErr::UsernameAlreadyUsed
                }
                err => {
                    error!("unexpected db error {err}");
                    DbUserUpdateUsernameErr::Db(err)
                }
            })
            .and_then_take_expect(0)
    }
}

#[tokio::test]
async fn test_user_username_change() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite2.id.key.clone(), 10, 10)
        .await
        .unwrap();

    assert_eq!(user.username, "hey");
    assert_eq!(user2.username, "hey2");

    let result = db.user_update_username(0, user.id.clone(), "hey2").await;
    assert_eq!(result, Err(DbUserUpdateUsernameErr::UsernameAlreadyUsed));

    let result = db
        .user_update_username(0, user.id.clone(), "hey3")
        .await
        .unwrap();
    assert_eq!(result, "hey3");
}
