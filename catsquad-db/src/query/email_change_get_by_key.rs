use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbEmailChange, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    create_email_change_id, create_invite_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbEmailChangeGetByKeyErr {
    #[error("email change not found")]
    EmailChangeNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn email_change_get_by_key(
        &self,
        time: u128,
        user_id: RecordId,
        email_change_key: impl Into<RecordIdKey>,
    ) -> Result<DbEmailChange, DbEmailChangeGetByKeyErr> {
        let email_change_id = create_email_change_id(email_change_key);
        let query = r#"
                BEGIN TRANSACTION;

                LET $email_change = SELECT *, user.* FROM ONLY $email_change_id;
                
                IF !$email_change {
                    THROW "not found"
                };

                IF $email_change.user.id != $user_id {
                    THROW "unauthorized"
                };

                IF $email_change.completed {
                    THROW "already used"
                };

                IF $email_change.expires < $time {
                    THROW "email change expired"
                };

                RETURN $email_change;

                COMMIT TRANSACTION;
            "#;

        // SELECT *, user.* FROM ONLY $email_change_id;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("email_change_id", email_change_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbEmailChangeGetByKeyErr::EmailChangeNotFound,
                err if err.thrown("unauthorized") => DbEmailChangeGetByKeyErr::Unauthorized,
                err if err.thrown("already used") => DbEmailChangeGetByKeyErr::AlreadyUsed,
                err if err.thrown("email change expired") => DbEmailChangeGetByKeyErr::Expired,
                err => {
                    error!("unexpected db error {err}");
                    DbEmailChangeGetByKeyErr::Db(err)
                }
            })
            .and_then_take_expect(6)
    }
}

#[tokio::test]
async fn test_email_change_get_by_key() {
    init_log();

    let db = Db::mem(0).await;

    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let user = db
        .user_add(0, "hey", "r4$$ohnGergnn023n", invite.id.key.clone(), 10, 10)
        .await
        .unwrap();
    let email_change = db.email_change_add(0, user.id.clone(), 10).await.unwrap();
    let _email_change = db
        .email_change_get_by_key(0, user.id.clone(), email_change.id.key.clone())
        .await
        .unwrap();
}
