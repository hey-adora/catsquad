use crate::{Db, DbPasswordChange, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPasswordChangeUpdateConfirmErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("expired")]
    Expired,

    #[error("already used")]
    AlreadyUsed,

    #[error("password key not found")]
    TokenNotFound,
}

impl Db {
    pub async fn password_change_update_confirm(
        &self,
        time: u64,
        password_change_token: Uuid,
        new_password: impl Into<String>,
    ) -> Result<String, DbPasswordChangeUpdateConfirmErr> {
        let pool = &self.db;
        let token = password_change_token;

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("password_change_update_confirm {err}"))?;

        // get password change
        let user_email = {
            let query = "SELECT
                                  password_change_user_email,
                                  password_change_used,
                                  password_change_expires_at
                               FROM passwords_changes WHERE password_change_token = $1";

            let result = sqlx::query_as(query)
                .bind(XUuid(token))
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let (user_email, used, XTimestamp(expires_at)): (String, bool, XTimestamp) =
                match result {
                    Ok(v) => v,
                    Err(sqlx::Error::RowNotFound) => {
                        return Err(DbPasswordChangeUpdateConfirmErr::TokenNotFound);
                    }
                    Err(err) => {
                        error!("unexpected db error {err}");
                        return Err(DbPasswordChangeUpdateConfirmErr::Db(err));
                    }
                };

            let expires_at = expires_at as u64;

            if expires_at < time {
                return Err(DbPasswordChangeUpdateConfirmErr::Expired);
            }

            if used {
                return Err(DbPasswordChangeUpdateConfirmErr::AlreadyUsed);
            }

            user_email
        };

        // delete sessoins
        {
            let query = "DELETE FROM sessions WHERE session_user_email = $1";

            let result = sqlx::query(query).bind(&user_email).execute(&mut *tx).await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPasswordChangeUpdateConfirmErr::Db(err));
                }
            };

            let affected = result.rows_affected();
            trace!("deleted sessions count {affected}");
        }

        // update user
        {
            let query = "UPDATE users SET
                    user_password = $1,
                    user_modified_at = $2
                    WHERE user_email = $3";

            let result = sqlx::query(query)
                .bind(new_password.into())
                .bind(XTimestamp(time as i64))
                .bind(&user_email)
                .execute(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let v = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPasswordChangeUpdateConfirmErr::Db(err));
                }
            };

            assert_eq!(v.rows_affected(), 1);
        }

        // update password change
        {
            let query = "UPDATE passwords_changes SET
                    password_change_used = TRUE,
                    password_change_modified_at = $1
                    WHERE password_change_token = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(XUuid(token))
                .execute(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let v = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPasswordChangeUpdateConfirmErr::Db(err));
                }
            };

            assert_eq!(v.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("password_change_update_confirm {err}"))?;

        Ok(user_email)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_password_change_update_confirm() {
    init_log();

    let db = Db::test_db(0, "test_password_change_update_confirm").await;
    let email = "hey@heyadora.com";

    let (user1, user2) = {
        let invite1 = db.invite_add(0, email, 1).await.unwrap();
        assert_eq!(invite1.email, email);

        let user = db
            .user_add(0, "hey", "hey", invite1.token, 10, 10)
            .await
            .unwrap();
        assert_eq!(user.password, "hey");

        let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey2", invite2.token, 10, 10)
            .await
            .unwrap();

        db.session_add(0, &user.email).await.unwrap();
        db.session_add(1, &user2.email).await.unwrap();
        let sessions = db.sessoin_get_all().await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].user_email, user2.email);
        assert_eq!(sessions[1].user_email, user.email);

        (user, user2)
    };

    let password_change = db.password_change_add(0, email, 10).await.unwrap();

    // assert expired
    {
        let result = db
            .password_change_update_confirm(11, password_change.token, "hey2")
            .await;
        assert!(matches!(
            result,
            Err(DbPasswordChangeUpdateConfirmErr::Expired)
        ));
    }

    // assert success
    {
        let result = db
            .password_change_update_confirm(0, password_change.token, "hey2")
            .await
            .unwrap();
        let user = db.user_get_by_email(email).await.unwrap();
        assert_eq!(result, user.email);
        assert_eq!(user.password, "hey2");
    }

    // assert that old sessions got deleted
    {
        let sessions = db.sessoin_get_all().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].user_email, user2.email);
    }

    // assert other errors
    {
        let result = db
            .password_change_update_confirm(0, password_change.token, "hey2")
            .await;
        assert!(matches!(
            result,
            Err(DbPasswordChangeUpdateConfirmErr::AlreadyUsed)
        ));

        let result = db
            .password_change_update_confirm(0, 0_u128.to_be_bytes(), "hey2")
            .await;
        assert!(matches!(
            result,
            Err(DbPasswordChangeUpdateConfirmErr::TokenNotFound)
        ));
    }
}
