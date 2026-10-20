use crate::{Db, DbTokenLabel, DbTokenUpdateConfirmErr, Uuid, XTimestamp};
use catsquad_log::prelude::*;

impl Db {
    pub async fn user_update_password_by_token(
        &self,
        time: u64,
        token_id: Uuid,
        new_password: impl Into<String>,
    ) -> Result<String, DbTokenUpdateConfirmErr> {
        let pool = &self.db;

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("token_update_confirm {err}"))?;

        let user_username = self
            .token_update_confirm_tx(&mut *tx, time, token_id, DbTokenLabel::PasswordChange)
            .await?;

        // delete sessoins
        {
            let query = "DELETE FROM sessions
                                    USING users
                                WHERE
                                    session_user_email=user_email
                                    AND user_username = $1";

            let result = sqlx::query(query)
                .bind(&user_username)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbTokenUpdateConfirmErr::Db(err));
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
                    WHERE user_username = $3";

            let result = sqlx::query(query)
                .bind(new_password.into())
                .bind(XTimestamp(time as i64))
                .bind(&user_username)
                .execute(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let v = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbTokenUpdateConfirmErr::Db(err));
                }
            };

            assert_eq!(v.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("password_change_update_confirm {err}"))?;

        Ok(user_username)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_update_password_by_token() {
    use crate::DbTokenLabel;

    init_log();

    let db = Db::test_db(0, "test_user_update_password_by_token").await;

    let (user1, user2) = {
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

        db.session_add(0, &user.email).await.unwrap();
        db.session_add(1, &user2.email).await.unwrap();
        let sessions = db.sessoin_get_all().await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].user_email, user2.email);
        assert_eq!(sessions[1].user_email, user.email);

        (user, user2)
    };

    let token = db
        .token_add(0, user1.username.clone(), 10, DbTokenLabel::PasswordChange)
        .await
        .unwrap();

    // assert expired
    {
        let result = db.user_update_password_by_token(11, token.id, "hey2").await;
        assert!(matches!(result, Err(DbTokenUpdateConfirmErr::Expired)));
    }

    // assert success
    {
        let result = db
            .user_update_password_by_token(0, token.id, "hey2")
            .await
            .unwrap();
        let user = db.user_get_by_email(user1.email.clone()).await.unwrap();
        assert_eq!(result, user.username);
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
        let result = db.user_update_password_by_token(0, token.id, "hey2").await;
        assert!(matches!(result, Err(DbTokenUpdateConfirmErr::AlreadyUsed)));

        let result = db
            .user_update_password_by_token(0, 0_u128.to_be_bytes(), "hey2")
            .await;
        assert!(matches!(
            result,
            Err(DbTokenUpdateConfirmErr::TokenNotFound)
        ));
    }
}
