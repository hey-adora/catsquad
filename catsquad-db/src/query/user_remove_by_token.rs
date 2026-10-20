use crate::{
    Db, DbTokenLabel, DbTokenUpdateConfirmErr, Uuid, query::user_remove_tx::DbUserRemoveErr,
};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserRemoveByTokenErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("unauthorized")]
    Unauthorized,

    #[error(transparent)]
    Image(#[from] DbUserRemoveErr),

    #[error(transparent)]
    Token(#[from] DbTokenUpdateConfirmErr),
}

impl Db {
    pub async fn user_remove_by_token(
        &self,
        time: u64,
        callback_remove_image: impl AsyncFnMut(i64, &str),
        token_id: Uuid,
        user_username: impl Into<String>,
    ) -> Result<(), DbUserRemoveByTokenErr> {
        let pool = &self.db;
        let user_username = user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("user_remove_by_token {err}"))?;

        let token_user_username = self
            .token_update_confirm_tx(&mut *tx, time, token_id, DbTokenLabel::AccountDeletion)
            .await?;

        if token_user_username != user_username {
            return Err(DbUserRemoveByTokenErr::Unauthorized);
        }

        self.user_remove(&mut *tx, time, callback_remove_image, token_user_username)
            .await?;

        tx.commit()
            .await
            .inspect_err(|err| error!("user_remove_by_token {err}"))?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_remove_by_token() {
    init_log();

    let db = Db::test_db(0, "test_user_remove_by_token").await;

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

    let callback_remove_image = async move |hash: i64, extension: &str| {
        //
    };

    // assert errors
    {
        let result = db
            .user_remove_by_token(
                0,
                callback_remove_image,
                0_u128.to_be_bytes(),
                user1.username.clone(),
            )
            .await;
        assert!(matches!(
            result,
            Err(DbUserRemoveByTokenErr::Token(
                DbTokenUpdateConfirmErr::TokenNotFound
            ))
        ));

        // not found
        {
            let result = db
                .user_remove_by_token(
                    0,
                    callback_remove_image,
                    0_u128.to_be_bytes(),
                    user1.username.clone(),
                )
                .await;

            assert!(matches!(
                result,
                Err(DbUserRemoveByTokenErr::Token(
                    DbTokenUpdateConfirmErr::TokenNotFound
                ))
            ));

            let token = db
                .token_add(0, user1.username.clone(), 10, DbTokenLabel::PasswordChange)
                .await
                .unwrap();

            let result = db
                .user_remove_by_token(0, callback_remove_image, token.id, user1.username.clone())
                .await;

            assert!(matches!(
                result,
                Err(DbUserRemoveByTokenErr::Token(
                    DbTokenUpdateConfirmErr::TokenNotFound
                ))
            ));
        }

        // error expired
        let token = {
            let token = db
                .token_add(0, user1.username.clone(), 10, DbTokenLabel::AccountDeletion)
                .await
                .unwrap();

            let result = db
                .user_remove_by_token(11, callback_remove_image, token.id, user1.username.clone())
                .await;

            assert!(matches!(
                result,
                Err(DbUserRemoveByTokenErr::Token(
                    DbTokenUpdateConfirmErr::Expired
                ))
            ));

            token
        };

        // unauthorized
        {
            let result = db
                .user_remove_by_token(0, callback_remove_image, token.id, user2.username.clone())
                .await;

            assert!(matches!(result, Err(DbUserRemoveByTokenErr::Unauthorized)));
        }

        // error already used
        {
            let mut tx = db.db.begin().await.unwrap();
            db.token_update_confirm_tx(&mut *tx, 0, token.id, DbTokenLabel::AccountDeletion)
                .await
                .unwrap();
            tx.commit().await.unwrap();

            let result = db
                .user_remove_by_token(0, callback_remove_image, token.id, user1.username.clone())
                .await;

            assert!(matches!(
                result,
                Err(DbUserRemoveByTokenErr::Token(
                    DbTokenUpdateConfirmErr::AlreadyUsed
                ))
            ));
        }
    }

    // assert success
    {
        let token = db
            .token_add(0, user1.username.clone(), 10, DbTokenLabel::AccountDeletion)
            .await
            .unwrap();

        let result = db
            .user_remove_by_token(0, callback_remove_image, token.id, user1.username.clone())
            .await;

        let users = db.user_get_all().await.unwrap();

        assert!(matches!(result, Ok(_)));
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "hey2");
    }
}
