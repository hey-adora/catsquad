use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbInvite {
    pub id: RecordId,
    pub email: String,
    pub expires: u128,
    pub used: bool,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbInviteAddErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),

    #[error("account with \"{0}\" email already exists")]
    EmailIsTaken(String),
}

pub fn create_invite_id(id: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("invite", id)
}

impl Db {
    pub async fn invite_define(&self) {
        let query = "
                DEFINE TABLE invite SCHEMAFULL;
                DEFINE FIELD email ON TABLE invite TYPE string;
                DEFINE FIELD expires ON TABLE invite TYPE number;
                DEFINE FIELD used ON TABLE invite TYPE bool;
                DEFINE FIELD modified_at ON TABLE invite TYPE number;
                DEFINE FIELD created_at ON TABLE invite TYPE number;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }
    pub async fn invite_add(
        &self,
        time: u128,
        email: impl Into<String>,
        expires: u128,
    ) -> Result<DbInvite, DbInviteAddErr> {
        let email: String = email.into();

        let query = r#"
                 BEGIN TRANSACTION;
                 LET $user_email = SELECT email FROM ONLY user WHERE email = $email;
                 IF $user_email {
                     THROW "email already used"
                 };
                 CREATE invite SET
                       kind = $kind,
                       email = $email,
                       expires = $expires,
                       used = false,
                       modified_at = $time,
                       created_at = $time
                       RETURN *;
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
                err if err.thrown("email already used") => DbInviteAddErr::EmailIsTaken(email),
                err => {
                    error!("unexpected db error {err}");
                    DbInviteAddErr::Db(err)
                }
            })
            .and_then_take_expect(3)
    }
}

#[tokio::test]
async fn test_invite_add() {
    init_log();

    let db = Db::mem().await;

    let invite1 = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    assert_eq!(invite1.email, "hey@hey.com");

    let _user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let invite2 = db.invite_add(0, "hey@hey.com", 1).await;
    assert_eq!(
        invite2,
        Err(DbInviteAddErr::EmailIsTaken("hey@hey.com".to_string()))
    );
}
