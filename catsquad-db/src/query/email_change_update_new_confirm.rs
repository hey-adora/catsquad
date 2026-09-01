use crate::{Db, DbEmailChange, DbUser, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbEmailChangeUpdateNewConfirmErr {
    #[error("email change not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("new email not set")]
    NewEmailNotSet,

    #[error("invalid token")]
    InvalidToken,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_change_update_new_confirm(
        &self,
        time: u64,
        user_username: impl Into<String>,
        email_change_id: i64,
        token: Uuid,
    ) -> Result<(), DbEmailChangeUpdateNewConfirmErr> {
        let pool = &self.db;

        let user_username = user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get email_change
        {
            let query = "SELECT
                                email_change_user_username,
                                email_change_completed,
                                email_change_expires_at,
                                email_change_new_used,
                                email_change_new_token,
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
                email_change_new_used,
                XUuid(email_chnged_new_token),
                email_change_new_email,
            ): (String, bool, XTimestamp, bool, XUuid, String) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbEmailChangeUpdateNewConfirmErr::NotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbEmailChangeUpdateNewConfirmErr::Db(err));
                }
            };
            let email_change_expired_at = email_change_expired_at as u64;

            if email_change_user_username != user_username {
                return Err(DbEmailChangeUpdateNewConfirmErr::Unauthorized);
            }

            if email_change_completed || email_change_new_used {
                return Err(DbEmailChangeUpdateNewConfirmErr::AlreadyUsed);
            }

            if email_change_expired_at < time {
                return Err(DbEmailChangeUpdateNewConfirmErr::Expired);
            }

            if email_change_new_email.is_empty() {
                return Err(DbEmailChangeUpdateNewConfirmErr::NewEmailNotSet);
            }

            if email_chnged_new_token != token {
                return Err(DbEmailChangeUpdateNewConfirmErr::InvalidToken);
            }
        }

        // update email change
        {
            let query = "UPDATE emails_changes SET
                                email_change_new_used = TRUE,
                                email_change_modified_at = $1
                                WHERE email_change_id = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(email_change_id)
                .execute(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbEmailChangeUpdateNewConfirmErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        Ok(())

        // let email_change_id = create_email_change_id(email_change_key);
        // let token = token.into();

        // let query = r#"
        //             BEGIN TRANSACTION;

        //             LET $email_change = SELECT *, new.*, current.* FROM ONLY $email_change_id;

        //             # basic checks

        //             IF !$email_change {
        //                 THROW "not found"
        //             };

        //             IF $email_change.user != $user_id {
        //                 THROW "unauthorized"
        //             };

        //             IF $email_change.new.token_used {
        //                 THROW "already used"
        //             };

        //             IF $email_change.expires < $time {
        //                 THROW "email change expired"
        //             };

        //             #

        //             IF !$email_change.new OR !$email_change.new.email {
        //                 THROW "new email not added"
        //             };

        //             #THROW [$email_change.new.token,  $token_new];
        //             IF !$token_new OR $email_change.new.token != $token_new {
        //                 THROW "invalid token"
        //             };

        //             UPDATE ONLY $email_change_id SET
        //                 new.token_used = true,
        //                 modified_at = $time
        //                 RETURN *, new.*, current.*, user.*;

        //             COMMIT TRANSACTION;
        //         "#;
        // trace!("about to run {query}");

        // self.db
        //     .query(query)
        //     .bind(("time", time))
        //     .bind(("token_new", token))
        //     .bind(("user_id", user_id))
        //     .bind(("email_change_id", email_change_id))
        //     .await
        //     .check_better(|err| match err {
        //         err if err.thrown("not found") => DbEmailChangeUpdateNewConfirmErr::NotFound,
        //         err if err.thrown("unauthorized") => DbEmailChangeUpdateNewConfirmErr::Unauthorized,
        //         err if err.thrown("already used") => DbEmailChangeUpdateNewConfirmErr::AlreadyUsed,
        //         err if err.thrown("email change expired") => {
        //             DbEmailChangeUpdateNewConfirmErr::Expired
        //         }
        //         err if err.thrown("new email not added") => {
        //             DbEmailChangeUpdateNewConfirmErr::NewEmailNotSet
        //         }
        //         err if err.thrown("invalid token") => {
        //             DbEmailChangeUpdateNewConfirmErr::InvalidToken
        //         }
        //         err => {
        //             error!("unexpected db error {err}");
        //             DbEmailChangeUpdateNewConfirmErr::Db(err)
        //         }
        //     })
        //     .and_then_take_expect(8)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_update_new_confirm() {
    init_log();

    let db = Db::test_db(0, "test_email_change_update_new_confirm").await;

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
            email_change.current_token.clone(),
        )
        .await
        .unwrap();

        db.email_change_update_new_add(
            0,
            user.username.clone(),
            email_change.id,
            "hey3@heyadora.com",
        )
        .await
        .unwrap();

        {
            let result = db
                .email_change_update_new_confirm(0, user.username.clone(), 0, 0_u128.to_be_bytes())
                .await;
            assert!(matches!(
                result,
                Err(DbEmailChangeUpdateNewConfirmErr::NotFound)
            ));

            let result = db
                .email_change_update_new_confirm(
                    0,
                    user2.username.clone(),
                    email_change.id,
                    0_u128.to_be_bytes(),
                )
                .await;
            assert!(matches!(
                result,
                Err(DbEmailChangeUpdateNewConfirmErr::Unauthorized)
            ));

            let result = db
                .email_change_update_new_confirm(
                    11,
                    user.username.clone(),
                    email_change.id,
                    email_change.new_token,
                )
                .await;
            assert!(matches!(
                result,
                Err(DbEmailChangeUpdateNewConfirmErr::Expired)
            ));

            let result = db
                .email_change_update_new_confirm(
                    9,
                    user.username.clone(),
                    email_change.id,
                    0_u128.to_be_bytes(),
                )
                .await;
            assert!(matches!(
                result,
                Err(DbEmailChangeUpdateNewConfirmErr::InvalidToken)
            ));
        }

        db.email_change_update_new_confirm(
            9,
            user.username.clone(),
            email_change.id,
            email_change.new_token,
        )
        .await
        .unwrap();

        let result = db
            .email_change_update_new_confirm(
                9,
                user.username.clone(),
                email_change.id,
                email_change.new_token,
            )
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateNewConfirmErr::AlreadyUsed)
        ));
    }

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
            .email_change_update_new_confirm(
                9,
                user.username.clone(),
                email_change.id,
                0_u128.to_be_bytes(),
            )
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateNewConfirmErr::NewEmailNotSet)
        ));
    }
}
