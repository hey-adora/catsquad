use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
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
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_change_update_finish(
        &self,
        time: u64,
        user_username: impl Into<String>,
        email_change_id: i64,
    ) -> Result<(), DbEmailChangeUpdateFinishErr> {
        let pool = &self.db;

        let user_username = user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get email_change
        let (email_change_current_email, email_change_new_email) = {
            let query = "SELECT
                                email_change_user_username,
                                email_change_current_email,
                                email_change_new_email,
                                email_change_completed,
                                email_change_expires_at,
                                email_change_new_used
                                FROM emails_changes WHERE email_change_id = $1";

            let result = sqlx::query_as(query)
                .bind(email_change_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let (
                email_change_user_username,
                email_change_current_email,
                email_change_new_email,
                email_change_completed,
                XTimestamp(email_change_expired_at),
                email_change_new_used,
            ): (String, String, String, bool, XTimestamp, bool) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbEmailChangeUpdateFinishErr::NotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbEmailChangeUpdateFinishErr::Db(err));
                }
            };
            let email_change_expired_at = email_change_expired_at as u64;

            if email_change_user_username != user_username {
                return Err(DbEmailChangeUpdateFinishErr::Unauthorized);
            }

            if email_change_completed {
                return Err(DbEmailChangeUpdateFinishErr::AlreadyUsed);
            }

            if email_change_expired_at < time {
                return Err(DbEmailChangeUpdateFinishErr::Expired);
            }

            if !email_change_new_used {
                return Err(DbEmailChangeUpdateFinishErr::NewEmailNotConfirmed);
            }

            (email_change_current_email, email_change_new_email)
        };

        // update email change
        {
            let query = "UPDATE emails_changes SET
                                email_change_completed = TRUE,
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
                    return Err(DbEmailChangeUpdateFinishErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        // update user
        {
            let query = "UPDATE users SET
                                user_email = $3,
                                user_modified_at = $1
                                WHERE user_email = $2 AND user_username = $4";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(email_change_current_email)
                .bind(email_change_new_email)
                .bind(user_username)
                .execute(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            // user_email_idx
            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::Database(err))
                    if err.is_unique_violation() && err.constraint() == Some("user_email_idx") =>
                {
                    return Err(DbEmailChangeUpdateFinishErr::EmailIsTaken);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbEmailChangeUpdateFinishErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_update_finish() {
    init_log();

    let db = Db::test_db(0, "test_email_change_update_finish").await;

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
        use crate::query::email_change_get_by_key::DbEmailChangeGetByIdErr;

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

        db.email_change_update_new_add(
            0,
            user.username.clone(),
            email_change.id,
            "hey3@heyadora.com",
        )
        .await
        .unwrap();

        db.email_change_update_new_confirm(
            0,
            user.username.clone(),
            email_change.id,
            email_change.new_token,
        )
        .await
        .unwrap();

        let result = db
            .email_change_update_finish(0, user.username.clone(), 0)
            .await;

        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::NotFound)
        ));

        let result = db
            .email_change_update_finish(0, user2.username.clone(), email_change.id)
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::Unauthorized)
        ));

        let result = db
            .email_change_update_finish(11, user.username.clone(), email_change.id)
            .await;
        assert!(matches!(result, Err(DbEmailChangeUpdateFinishErr::Expired)));

        db.email_change_update_finish(0, user.username.clone(), email_change.id)
            .await
            .unwrap();
        let result = db
            .email_change_get_by_id(0, user.username.clone(), email_change.id)
            .await;
        // .unwrap();
        let user = db
            .user_get_by_username(user.username.clone())
            .await
            .unwrap();

        assert!(matches!(result, Err(DbEmailChangeGetByIdErr::AlreadyUsed)));
        assert_eq!(user.email, "hey3@heyadora.com");

        let result = db
            .email_change_update_finish(0, user.username.clone(), email_change.id)
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::AlreadyUsed)
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

        db.email_change_update_new_add(
            0,
            user.username.clone(),
            email_change.id,
            "hey4@heyadora.com",
        )
        .await
        .unwrap();

        let result = db
            .email_change_update_finish(0, user.username.clone(), email_change.id)
            .await;
        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::NewEmailNotConfirmed)
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

        db.email_change_update_new_add(
            0,
            user.username.clone(),
            email_change.id,
            "hey5@heyadora.com",
        )
        .await
        .unwrap();

        db.email_change_update_new_confirm(
            0,
            user.username.clone(),
            email_change.id,
            email_change.new_token,
        )
        .await
        .unwrap();

        let invite5 = db.invite_add(0, "hey5@heyadora.com", 1).await.unwrap();
        let _user5 = db
            .user_add(0, "hey5", "hey5", invite5.token, 10, 10)
            .await
            .unwrap();

        let result = db
            .email_change_update_finish(0, user.username.clone(), email_change.id)
            .await;

        assert!(matches!(
            result,
            Err(DbEmailChangeUpdateFinishErr::EmailIsTaken)
        ));
    }
}
