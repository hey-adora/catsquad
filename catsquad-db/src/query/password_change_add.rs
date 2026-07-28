use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbPasswordChange {
    pub id: RecordId,
    pub user: DbUser,
    pub expires: u128,
    pub used: bool,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPasswordChangeAddErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),

    #[error("user with \"{0}\" email not found")]
    UserNotFound(String),
}

pub fn create_password_change_id(key: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("password_change", key)
}

impl Db {
    pub async fn password_change_define(&self) {
        let query = "
                DEFINE TABLE password_change SCHEMAFULL;
                DEFINE FIELD user ON TABLE password_change TYPE record<user>;
                DEFINE FIELD expires ON TABLE password_change TYPE number;
                DEFINE FIELD used ON TABLE password_change TYPE bool;
                DEFINE FIELD modified_at ON TABLE password_change TYPE number;
                DEFINE FIELD created_at ON TABLE password_change TYPE number;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn password_change_add(
        &self,
        time: u128,
        email: impl Into<String>,
        expires: u128,
    ) -> Result<DbPasswordChange, DbPasswordChangeAddErr> {
        let email: String = email.into();

        let query = r#"
                 BEGIN TRANSACTION;
                 LET $user = SELECT id FROM ONLY user WHERE email = $email;
                 CREATE password_change SET
                       user = $user.id,
                       expires = $expires,
                       used = false,
                       modified_at = $time,
                       created_at = $time
                       RETURN *, user.*;
                COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("email", email.clone()))
            .bind(("expires", expires))
            .await
            .check_better(|err| match err {
                err if err.field_value_null("user") => DbPasswordChangeAddErr::UserNotFound(email),
                err => {
                    error!("unexpected db error {err}");
                    DbPasswordChangeAddErr::Db(err)
                }
            })
            .and_then_take_expect(2)
    }
}

#[tokio::test]
async fn test_password_change_add() {
    init_log();

    let db = Db::mem(0).await;

    let email = "hey@heyadora.com";
    let invite1 = db.invite_add(0, email, 1).await.unwrap();
    assert_eq!(invite1.email, email);

    let _user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let _result = db.password_change_add(0, email, 10).await.unwrap();

    let result = db.password_change_add(0, "invalid", 10).await;
    assert_eq!(
        result,
        Err(DbPasswordChangeAddErr::UserNotFound("invalid".to_string()))
    );
}
