use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{
    Db, DbPasswordChange, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    create_password_change_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPasswordChangeUpdateConfirmErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),

    #[error("expired")]
    Expired,

    #[error("already used")]
    AlreadyUsed,

    #[error("password key not found")]
    PasswordKeyNotFound,
}

impl Db {
    pub async fn password_change_update_confirm(
        &self,
        time: u128,
        password_change_key: impl Into<RecordIdKey>,
        new_password: impl Into<String>,
    ) -> Result<DbPasswordChange, DbPasswordChangeUpdateConfirmErr> {
        let password_change_id = create_password_change_id(password_change_key);
        let new_password = new_password.into();

        let query = r#"
                 BEGIN TRANSACTION;
                 LET $password_change = SELECT used, expires, user FROM ONLY $password_change_id;
                 if !$password_change {
                     THROW "password change not found"
                 };
                 if $password_change.used {
                     THROW "password change already used"
                 };
                 if $password_change.expires < $time {
                     THROW "password change expired"
                 };
                 UPDATE $password_change.user SET password = $new_password;
                 UPDATE $password_change_id SET used = true RETURN *, user.*;
                 DELETE session WHERE user = $password_change.user;
                 COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("password_change_id", password_change_id))
            .bind(("new_password", new_password))
            // .bind(("expires", expires))
            .await
            .check_better(|err| match err {
                err if err.thrown("password change not found") => {
                    DbPasswordChangeUpdateConfirmErr::PasswordKeyNotFound
                }
                err if err.thrown("password change already used") => {
                    DbPasswordChangeUpdateConfirmErr::AlreadyUsed
                }
                err if err.thrown("password change expired") => {
                    DbPasswordChangeUpdateConfirmErr::Expired
                }
                err => {
                    error!("unexpected db error {err}");
                    DbPasswordChangeUpdateConfirmErr::Db(err)
                }
            })
            .and_then_take_expect(6)
    }
}

#[tokio::test]
async fn test_password_change_update_confirm() {
    init_log();

    let db = Db::mem(0).await;

    let email = "hey@heyadora.com";
    let invite1 = db.invite_add(0, email, 1).await.unwrap();
    assert_eq!(invite1.email, email);

    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();
    assert_eq!(user.password, "hey");

    let password_change = db.password_change_add(0, email, 10).await.unwrap();

    let result = db
        .password_change_update_confirm(11, password_change.id.key.clone(), "hey2")
        .await;
    assert_eq!(result, Err(DbPasswordChangeUpdateConfirmErr::Expired));

    db.password_change_update_confirm(0, password_change.id.key.clone(), "hey2")
        .await
        .unwrap();
    let user = db.user_get_by_email(email).await.unwrap();
    assert_eq!(user.password, "hey2");

    let result = db
        .password_change_update_confirm(0, password_change.id.key.clone(), "hey2")
        .await;
    assert_eq!(result, Err(DbPasswordChangeUpdateConfirmErr::AlreadyUsed));

    let result = db
        .password_change_update_confirm(0, "invalid", "hey2")
        .await;
    assert_eq!(
        result,
        Err(DbPasswordChangeUpdateConfirmErr::PasswordKeyNotFound)
    );
}
