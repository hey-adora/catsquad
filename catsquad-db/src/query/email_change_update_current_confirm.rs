use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_user_id,
    query::email_change_add::{DbEmailChange, create_email_change_id},
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbEmailChangeConfirmUpdateCurrentErr {
    #[error("email change not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("invalid token")]
    InvalidToken,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn email_change_update_current_confirm(
        &self,
        time: u128,
        user_id: RecordId,
        email_change_key: impl Into<RecordIdKey>,
        token: impl Into<String>,
    ) -> Result<DbEmailChange, DbEmailChangeConfirmUpdateCurrentErr> {
        // let user_id = create_user_id(user_key);
        let email_change_id = create_email_change_id(email_change_key);
        let token = token.into();

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $email_change = SELECT user, current.token, current.token_used, expires FROM ONLY $email_change_id;

                    # basic checks

                    IF !$email_change {
                        THROW "not found"
                    };

                    IF $email_change.user != $user_id {
                        THROW "unauthorized"
                    };

                    IF $email_change.current.token_used {
                        THROW "already used"
                    };

                    IF $email_change.expires < $time {
                        THROW "email change expired"
                    };

                    #

                    IF $email_change.current.token != $token_current {
                        THROW "invalid token"
                    };

                    UPDATE $email_change_id SET
                        current.token_used = true,
                        modified_at = $time
                        RETURN *, user.*;

                    COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("token_current", token))
            .bind(("user_id", user_id))
            .bind(("email_change_id", email_change_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbEmailChangeConfirmUpdateCurrentErr::NotFound,
                err if err.thrown("unauthorized") => {
                    DbEmailChangeConfirmUpdateCurrentErr::Unauthorized
                }
                err if err.thrown("already used") => {
                    DbEmailChangeConfirmUpdateCurrentErr::AlreadyUsed
                }
                err if err.thrown("email change expired") => {
                    DbEmailChangeConfirmUpdateCurrentErr::Expired
                }
                err if err.thrown("invalid token") => {
                    DbEmailChangeConfirmUpdateCurrentErr::InvalidToken
                }
                err => {
                    error!("unexpected db error {err}");
                    DbEmailChangeConfirmUpdateCurrentErr::Db(err)
                }
            })
            .and_then_take_expect(7)
    }
}

#[tokio::test]
async fn test_email_change_confirm_update_current() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey2", invite2.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let email_change = db.email_change_add(0, user.id.clone(), 10).await.unwrap();

    let result = db
        .email_change_update_current_confirm(0, user.id.clone(), "invalid", "invalid")
        .await;
    assert_eq!(result, Err(DbEmailChangeConfirmUpdateCurrentErr::NotFound));

    let result = db
        .email_change_update_current_confirm(0, user.id.clone(), "", "")
        .await;
    assert_eq!(result, Err(DbEmailChangeConfirmUpdateCurrentErr::NotFound));

    let result = db
        .email_change_update_current_confirm(
            0,
            user2.id.clone(),
            email_change.id.key.clone(),
            email_change.current.token.clone(),
        )
        .await;
    assert_eq!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::Unauthorized)
    );

    let result = db
        .email_change_update_current_confirm(
            11,
            user.id.clone(),
            email_change.id.key.clone(),
            email_change.current.token.clone(),
        )
        .await;
    assert_eq!(result, Err(DbEmailChangeConfirmUpdateCurrentErr::Expired));

    let result = db
        .email_change_update_current_confirm(0, user.id.clone(), email_change.id.key.clone(), "")
        .await;
    assert_eq!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::InvalidToken)
    );

    let result = db
        .email_change_update_current_confirm(
            0,
            user.id.clone(),
            email_change.id.key.clone(),
            email_change.current.token.clone(),
        )
        .await;
    assert!(matches!(result, Ok(_)));

    let result = db
        .email_change_update_current_confirm(
            0,
            user.id.clone(),
            email_change.id.key.clone(),
            email_change.current.token.clone(),
        )
        .await;
    assert_eq!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::AlreadyUsed)
    );
}
