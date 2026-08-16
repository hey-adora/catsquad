use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_user_id,
    query::email_change_add::{DbEmailChange, create_email_change_id},
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbEmailChangeUpdateNewAddErr {
    #[error("email change not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("current email not confirmed")]
    NotConfirmed,

    #[error("email {0} already taken")]
    EmailIsTaken(String),

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn email_change_update_new_add(
        &self,
        time: u128,
        user_id: RecordId,
        email_change_key: impl Into<RecordIdKey>,
        new_email: impl Into<String>,
    ) -> Result<DbEmailChange, DbEmailChangeUpdateNewAddErr> {
        // let user_id = create_user_id(user_key);
        let email_change_id = create_email_change_id(email_change_key);
        let new_email = new_email.into();
        let token_new = RecordIdKey::rand().to_sql();

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $email_change = SELECT user, current.token_used, new.token_used, new.email, expires, completed FROM ONLY $email_change_id;

                    # basic check

                    IF !$email_change {
                        THROW "not found"
                    };

                    IF $email_change.user != $user_id {
                        THROW "unauthorized"
                    };

                    IF $email_change.new.email {
                        THROW "already used"
                    };

                    IF $email_change.expires < $time {
                        THROW "email change expired"
                    };

                    #

                    IF !$email_change.current.token_used {
                        THROW "current email not confirmed"
                    };

                    IF (SELECT NONE FROM ONLY user WHERE email = $new_email) {
                        THROW "email is taken"
                    };

                    UPDATE $email_change_id SET
                        new.email = $new_email,
                        new.token = $token_new,
                        new.token_used = false,
                        modified_at = $time
                        RETURN *, user.*;

                    COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("token_new", token_new))
            .bind(("new_email", new_email.clone()))
            .bind(("user_id", user_id))
            .bind(("email_change_id", email_change_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbEmailChangeUpdateNewAddErr::NotFound,
                err if err.thrown("unauthorized") => DbEmailChangeUpdateNewAddErr::Unauthorized,
                err if err.thrown("already used") => DbEmailChangeUpdateNewAddErr::AlreadyUsed,
                err if err.thrown("current email not confirmed") => {
                    DbEmailChangeUpdateNewAddErr::NotConfirmed
                }
                err if err.thrown("email change expired") => DbEmailChangeUpdateNewAddErr::Expired,
                err if err.thrown("email is taken") => {
                    DbEmailChangeUpdateNewAddErr::EmailIsTaken(new_email)
                }
                err => {
                    error!("unexpected db error {err}");
                    DbEmailChangeUpdateNewAddErr::Db(err)
                }
            })
            .and_then_take_expect(8)
    }
}

#[tokio::test]
async fn test_email_change_update_new_add() {
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

        let result = db
            .email_change_update_new_add(0, user.id.clone(), "", "hey2@heyadora.com")
            .await;
        assert_eq!(result, Err(DbEmailChangeUpdateNewAddErr::NotFound));

        let result = db
            .email_change_update_new_add(
                0,
                user2.id.clone(),
                email_change.id.key.clone(),
                "hey2@heyadora.com",
            )
            .await;
        assert_eq!(result, Err(DbEmailChangeUpdateNewAddErr::Unauthorized));

        let result = db
            .email_change_update_new_add(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                "hey2@heyadora.com",
            )
            .await;
        assert_eq!(
            result,
            Err(DbEmailChangeUpdateNewAddErr::EmailIsTaken(
                "hey2@heyadora.com".to_string()
            ))
        );

        let result = db
            .email_change_update_new_add(
                30,
                user.id.clone(),
                email_change.id.key.clone(),
                "hey3@heyadora.com",
            )
            .await;
        assert_eq!(result, Err(DbEmailChangeUpdateNewAddErr::Expired));

        let email_change = db
            .email_change_update_new_add(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                "hey3@heyadora.com",
            )
            .await
            .unwrap();
        assert_eq!(email_change.new.clone().unwrap().token.len(), 20);

        let result = db
            .email_change_update_new_add(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                "hey3@heyadora.com",
            )
            .await;
        assert_eq!(result, Err(DbEmailChangeUpdateNewAddErr::AlreadyUsed));
    }

    {
        let email_change = db.email_change_add(0, user.id.clone(), 10).await.unwrap();

        let result = db
            .email_change_update_new_add(
                0,
                user.id.clone(),
                email_change.id.key.clone(),
                "hey2@heyadora.com",
            )
            .await;
        assert_eq!(result, Err(DbEmailChangeUpdateNewAddErr::NotConfirmed));
    }
}
