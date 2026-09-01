use crate::{Db, DbUser, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
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
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_change_update_current_confirm(
        &self,
        time: u64,
        user_username: impl Into<String>,
        email_change_id: i64,
        token: Uuid,
    ) -> Result<(), DbEmailChangeConfirmUpdateCurrentErr> {
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
                                email_change_current_used,
                                email_change_current_token
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
                XUuid(email_chnged_current_token),
            ): (String, bool, XTimestamp, bool, XUuid) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbEmailChangeConfirmUpdateCurrentErr::NotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbEmailChangeConfirmUpdateCurrentErr::Db(err));
                }
            };
            let email_change_expired_at = email_change_expired_at as u64;

            if email_change_user_username != user_username {
                return Err(DbEmailChangeConfirmUpdateCurrentErr::Unauthorized);
            }

            if email_change_completed || email_change_current_used {
                return Err(DbEmailChangeConfirmUpdateCurrentErr::AlreadyUsed);
            }

            if email_change_expired_at < time {
                return Err(DbEmailChangeConfirmUpdateCurrentErr::Expired);
            }

            if email_chnged_current_token != token {
                return Err(DbEmailChangeConfirmUpdateCurrentErr::InvalidToken);
            }
        }

        // update email change
        {
            let query = "UPDATE emails_changes SET
                                email_change_current_used = TRUE,
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
                    return Err(DbEmailChangeConfirmUpdateCurrentErr::Db(err));
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
async fn test_email_change_confirm_update_current() {
    init_log();

    let db = Db::test_db(0, "test_email_change_confirm_update_current").await;

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

    let email_change = db
        .email_change_add(0, user.username.clone(), 10)
        .await
        .unwrap();

    let result = db
        .email_change_update_current_confirm(0, user.username.clone(), 0, 0_u128.to_be_bytes())
        .await;
    assert!(matches!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::NotFound)
    ));

    let result = db
        .email_change_update_current_confirm(0, user.username.clone(), 0, 0_u128.to_be_bytes())
        .await;
    assert!(matches!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::NotFound)
    ));

    let result = db
        .email_change_update_current_confirm(
            0,
            user2.username.clone(),
            email_change.id,
            email_change.current_token,
        )
        .await;

    assert!(matches!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::Unauthorized)
    ));

    let result = db
        .email_change_update_current_confirm(
            11,
            user.username.clone(),
            email_change.id,
            email_change.current_token,
        )
        .await;
    assert!(matches!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::Expired)
    ));

    let result = db
        .email_change_update_current_confirm(
            0,
            user.username.clone(),
            email_change.id,
            0_u128.to_be_bytes(),
        )
        .await;
    assert!(matches!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::InvalidToken)
    ));

    let result = db
        .email_change_update_current_confirm(
            0,
            user.username.clone(),
            email_change.id,
            email_change.current_token,
        )
        .await;
    assert!(matches!(result, Ok(_)));

    let result = db
        .email_change_update_current_confirm(
            0,
            user.username.clone(),
            email_change.id,
            email_change.current_token,
        )
        .await;
    assert!(matches!(
        result,
        Err(DbEmailChangeConfirmUpdateCurrentErr::AlreadyUsed)
    ));
}
