use crate::{Db, DbUser, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
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
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_change_update_new_add(
        &self,
        time: u64,
        user_username: impl Into<String>,
        email_change_id: i64,
        new_email: impl Into<String>,
    ) -> Result<(), DbEmailChangeUpdateNewAddErr> {
        let pool = &self.db;
        // let user_id = create_user_id(user_key);
        let user_username = user_username.into();
        let new_email = new_email.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get user
        {
            let query = "SELECT EXISTS(SELECT 1 FROM users WHERE user_email=$1)";

            let result = sqlx::query_as(query)
                .bind(&new_email)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            match result {
                Ok((false,)) => (),
                Ok((true,)) => return Err(DbEmailChangeUpdateNewAddErr::EmailIsTaken(new_email)),
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbEmailChangeUpdateNewAddErr::Db(err));
                }
            };
        }

        // get email_change
        {
            let query = "SELECT
                                email_change_user_username,
                                email_change_completed,
                                email_change_expires_at,
                                email_change_current_used,
                                email_change_new_email
                                FROM emails_changes WHERE email_change_id = $1";

            let result = sqlx::query_as(query)
                .bind(email_change_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let (
                email_change_user_username,
                email_change_completed,
                XTimestamp(email_change_expired_at),
                email_change_current_used,
                email_change_new_email,
            ): (String, bool, XTimestamp, bool, String) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbEmailChangeUpdateNewAddErr::NotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbEmailChangeUpdateNewAddErr::Db(err));
                }
            };
            let email_change_expired_at = email_change_expired_at as u64;

            if email_change_user_username != user_username {
                return Err(DbEmailChangeUpdateNewAddErr::Unauthorized);
            }

            if email_change_completed || !email_change_new_email.is_empty() {
                return Err(DbEmailChangeUpdateNewAddErr::AlreadyUsed);
            }

            if email_change_expired_at < time {
                return Err(DbEmailChangeUpdateNewAddErr::Expired);
            }

            if !email_change_current_used {
                return Err(DbEmailChangeUpdateNewAddErr::NotConfirmed);
            }
        }

        // update email change
        {
            let query = "UPDATE emails_changes SET
                                email_change_new_email = $3,
                                email_change_modified_at = $1
                                WHERE email_change_id = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(email_change_id)
                .bind(new_email)
                .execute(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbEmailChangeUpdateNewAddErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        Ok(())

        // let query = r#"
        //             BEGIN TRANSACTION;

        //             LET $email_change = SELECT user, current.token_used, new.token_used, new.email, expires, completed FROM ONLY $email_change_id;

        //             # basic check

        //             IF !$email_change {
        //                 THROW "not found"
        //             };

        //             IF $email_change.user != $user_id {
        //                 THROW "unauthorized"
        //             };

        //             IF $email_change.new.email {
        //                 THROW "already used"
        //             };

        //             IF $email_change.expires < $time {
        //                 THROW "email change expired"
        //             };

        //             #

        //             IF !$email_change.current.token_used {
        //                 THROW "current email not confirmed"
        //             };

        //             IF (SELECT NONE FROM ONLY user WHERE email = $new_email) {
        //                 THROW "email is taken"
        //             };

        //             UPDATE $email_change_id SET
        //                 new.email = $new_email,
        //                 new.token = $token_new,
        //                 new.token_used = false,
        //                 modified_at = $time
        //                 RETURN *, user.*;

        //             COMMIT TRANSACTION;
        //         "#;
        // trace!("about to run {query}");

        // self.db
        //     .query(query)
        //     .bind(("time", time))
        //     .bind(("token_new", token_new))
        //     .bind(("new_email", new_email.clone()))
        //     .bind(("user_id", user_id))
        //     .bind(("email_change_id", email_change_id))
        //     .await
        //     .check_better(|err| match err {
        //         err if err.thrown("not found") => DbEmailChangeUpdateNewAddErr::NotFound,
        //         err if err.thrown("unauthorized") => DbEmailChangeUpdateNewAddErr::Unauthorized,
        //         err if err.thrown("already used") => DbEmailChangeUpdateNewAddErr::AlreadyUsed,
        //         err if err.thrown("current email not confirmed") => {
        //             DbEmailChangeUpdateNewAddErr::NotConfirmed
        //         }
        //         err if err.thrown("email change expired") => DbEmailChangeUpdateNewAddErr::Expired,
        //         err if err.thrown("email is taken") => {
        //             DbEmailChangeUpdateNewAddErr::EmailIsTaken(new_email)
        //         }
        //         err => {
        //             error!("unexpected db error {err}");
        //             DbEmailChangeUpdateNewAddErr::Db(err)
        //         }
        //     })
        //     .and_then_take_expect(8)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_update_new_add() {
    init_log();

    let db = Db::test_db(0, "test_email_change_update_new_add").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey2", invite2.token, 10, 10)
        .await
        .unwrap();

    {
        let email_change = db
            .email_change_add(0, user.username.clone(), 10)
            .await
            .unwrap();

        db.email_change_update_current_confirm(
            0,
            user.username.clone(),
            email_change.id,
            email_change.current_token,
        )
        .await
        .unwrap();

        let result = db
            .email_change_update_new_add(0, user.username.clone(), 0, "hey3@heyadora.com")
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateNewAddErr::NotFound)
        ));

        let result = db
            .email_change_update_new_add(
                0,
                user2.username.clone(),
                email_change.id,
                "hey3@heyadora.com",
            )
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateNewAddErr::Unauthorized)
        ));

        let result = db
            .email_change_update_new_add(
                0,
                user.username.clone(),
                email_change.id,
                "hey2@heyadora.com",
            )
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateNewAddErr::EmailIsTaken(_))
        ));

        let result = db
            .email_change_update_new_add(
                30,
                user.username.clone(),
                email_change.id,
                "hey3@heyadora.com",
            )
            .await;
        assert!(matches!(result, Err(DbEmailChangeUpdateNewAddErr::Expired)));

        db.email_change_update_new_add(
            0,
            user.username.clone(),
            email_change.id,
            "hey3@heyadora.com",
        )
        .await
        .unwrap();
        assert!(email_change.new_token.len() > 10);

        let result = db
            .email_change_update_new_add(
                0,
                user.username.clone(),
                email_change.id,
                "hey3@heyadora.com",
            )
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateNewAddErr::AlreadyUsed)
        ));
    }

    {
        let email_change = db
            .email_change_add(0, user.username.clone(), 10)
            .await
            .unwrap();

        let result = db
            .email_change_update_new_add(
                0,
                user.username.clone(),
                email_change.id,
                "hey4@heyadora.com",
            )
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateNewAddErr::NotConfirmed)
        ));
    }
}
