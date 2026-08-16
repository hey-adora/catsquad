use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_user_id,
    query::email_change_add::{DbEmailChange, create_email_change_id},
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbEmailChangeUpdateFinishErr {
    #[error("email change not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("new email not confirmed")]
    NewEmailNotConfirmed,

    #[error("email already taken")]
    EmailIsTaken,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn email_change_update_finish(
        &self,
        time: u128,
        user_id: RecordId,
        email_change_key: impl Into<RecordIdKey>,
    ) -> Result<DbEmailChange, DbEmailChangeUpdateFinishErr> {
        // let user_id = create_user_id(user_key);
        let email_change_id = create_email_change_id(email_change_key);

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $email_change = SELECT *, new.*, current.* FROM ONLY $email_change_id;

                    # basic checks
                    
                    IF !$email_change {
                        THROW "not found"
                    };

                    IF $email_change.user != $user_id {
                        THROW "unauthorized"
                    };

                    IF $email_change.completed {
                        THROW "already used"
                    };

                    IF $email_change.expires < $time {
                        THROW "email change expired"
                    };

                    # 

                    IF !$email_change.new.token_used {
                        THROW "previous step not complete"
                    };

                    UPDATE ONLY $user_id SET email = $email_change.new.email RETURN NONE;

                    UPDATE ONLY $email_change_id SET
                        completed = true,
                        modified_at = $time
                        RETURN *, new.*, current.*, user.*;

                    COMMIT TRANSACTION;
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("email_change_id", email_change_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbEmailChangeUpdateFinishErr::NotFound,
                err if err.thrown("unauthorized") => DbEmailChangeUpdateFinishErr::Unauthorized,
                err if err.thrown("already used") => DbEmailChangeUpdateFinishErr::AlreadyUsed,
                err if err.thrown("email change expired") => DbEmailChangeUpdateFinishErr::Expired,
                err if err.thrown("previous step not complete") => {
                    DbEmailChangeUpdateFinishErr::NewEmailNotConfirmed
                }
                err if err.index_exists("idx_user_email") => {
                    DbEmailChangeUpdateFinishErr::EmailIsTaken
                }
                err => {
                    error!("unexpected db error {err}");
                    DbEmailChangeUpdateFinishErr::Db(err)
                }
            })
            .and_then_take_expect(8)
    }
}

#[tokio::test]
async fn test_email_change_update_finish() {
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

    {
        let email_change = db.email_change_add(0, user.id.clone(), 10).await.unwrap();

        let email_change = db
            .email_change_update_current_confirm(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                email_change.current.token.clone(),
            )
            .await
            .unwrap();

        let email_change = db
            .email_change_update_new_add(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                "hey3@heyadora.com",
            )
            .await
            .unwrap();

        let email_change = db
            .email_change_update_new_confirm(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                email_change.new.clone().unwrap().token,
            )
            .await
            .unwrap();

        let result = db.email_change_update_finish(0, user.id.clone(), "").await;

        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::NotFound)
        ));

        let result = db
            .email_change_update_finish(0, user2.id.clone(), email_change.id.key.clone())
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::Unauthorized)
        ));

        let result = db
            .email_change_update_finish(11, user.id.clone(), email_change.id.key.clone())
            .await;
        assert!(matches!(result, Err(DbEmailChangeUpdateFinishErr::Expired)));

        let email_change = db
            .email_change_update_finish(0, user.id.clone(), email_change.id.key.clone())
            .await
            .unwrap();

        assert!(email_change.completed);
        assert_eq!(email_change.user.email, "hey3@heyadora.com");

        let result = db
            .email_change_update_finish(0, user.id.clone(), email_change.id.key.clone())
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::AlreadyUsed)
        ));
    }

    {
        let email_change = db.email_change_add(0, user.id.clone(), 10).await.unwrap();

        let email_change = db
            .email_change_update_current_confirm(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                email_change.current.token.clone(),
            )
            .await
            .unwrap();

        let email_change = db
            .email_change_update_new_add(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                "hey4@heyadora.com",
            )
            .await
            .unwrap();

        let result = db
            .email_change_update_finish(0, user.id.clone(), email_change.id.key.clone())
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::NewEmailNotConfirmed)
        ));
    }

    {
        let email_change = db.email_change_add(0, user.id.clone(), 10).await.unwrap();

        let email_change = db
            .email_change_update_current_confirm(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                email_change.current.token.clone(),
            )
            .await
            .unwrap();

        let email_change = db
            .email_change_update_new_add(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                "hey5@heyadora.com",
            )
            .await
            .unwrap();

        let email_change = db
            .email_change_update_new_confirm(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                email_change.new.clone().unwrap().token,
            )
            .await
            .unwrap();

        let invite5 = db.invite_add(0, "hey5@heyadora.com", 1).await.unwrap();
        let _user5 = db
            .user_add(0, "hey5", "hey5", invite5.id.key.clone(), 10, 10)
            .await
            .unwrap();

        let result = db
            .email_change_update_finish(0, user.id.clone(), email_change.id.key.clone())
            .await;

        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::EmailIsTaken)
        ));
    }
}
