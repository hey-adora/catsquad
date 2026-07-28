use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_user_id,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbEmailChange {
    pub id: RecordId,
    pub user: DbUser,
    pub current: DbEmailChangeToken,
    pub new: Option<DbEmailChangeToken>,
    pub completed: bool,
    pub expires: u128,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbEmailChangeToken {
    pub email: String,
    pub token: String,
    pub token_used: bool,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbEmailChangeAddErr {
    #[error("user not found")]
    UserNotFound,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

pub fn create_email_change_id(key: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("email_change", key)
}

impl Db {
    pub async fn email_change_define(&self) {
        let query = "
                DEFINE TABLE email_change SCHEMAFULL;
                DEFINE FIELD user ON TABLE email_change TYPE record<user>;

                DEFINE FIELD current ON TABLE email_change TYPE object;
                DEFINE FIELD current.email ON TABLE email_change TYPE string;
                DEFINE FIELD current.token ON TABLE email_change TYPE string;
                DEFINE FIELD current.token_used ON TABLE email_change TYPE bool;

                DEFINE FIELD new ON TABLE email_change TYPE option<object>;
                DEFINE FIELD new.email ON TABLE email_change TYPE string;
                DEFINE FIELD new.token ON TABLE email_change TYPE string;
                DEFINE FIELD new.token_used ON TABLE email_change TYPE bool;

                DEFINE FIELD completed ON TABLE email_change TYPE bool;
                DEFINE FIELD expires ON TABLE email_change TYPE number;
                DEFINE FIELD modified_at ON TABLE email_change TYPE number;
                DEFINE FIELD created_at ON TABLE email_change TYPE number;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn email_change_add(
        &self,
        time: u128,
        user_id: RecordId,
        expires: u128,
    ) -> Result<DbEmailChange, DbEmailChangeAddErr> {
        let token_current = RecordIdKey::rand().to_sql();

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $user = SELECT id, email FROM ONLY $user_id;

                    if !$user {
                        THROW "user not found"
                    };

                    CREATE email_change SET
                       user = $user.id,
                       current.email = $user.email,
                       current.token = $token_current,
                       current.token_used = false,
                       new = NONE,
                       completed = false,
                       expires = $expires,
                       modified_at = $time,
                       created_at = $time 
                    RETURN *, user.*;

                    COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("expires", expires))
            .bind(("token_current", token_current))
            .bind(("user_id", user_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("user not found") => DbEmailChangeAddErr::UserNotFound,
                err => {
                    error!("unexpected db error {err}");
                    DbEmailChangeAddErr::Db(err)
                }
            })
            .and_then_take_expect(3)
    }
}

#[tokio::test]
async fn test_email_change_add() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let _result = db.email_change_add(0, user.id.clone(), 10).await.unwrap();

    let result = db.email_change_add(0, create_user_id("invalid"), 10).await;
    assert_eq!(result, Err(DbEmailChangeAddErr::UserNotFound));
}
