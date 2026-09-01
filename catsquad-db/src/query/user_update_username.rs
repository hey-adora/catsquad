use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdateUsernameErr {
    #[error("username already used")]
    UsernameAlreadyUsed,

    #[error("user not found")]
    UserNotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn user_update_username(
        &self,
        time: u64,
        user_username: impl Into<String>,
        new_username: impl Into<String>,
    ) -> Result<(), DbUserUpdateUsernameErr> {
        let pool = &self.db;

        let query = "UPDATE users SET
                            user_username = $3,
                            user_modified_at = $1
                           WHERE user_username = $2";

        debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(XTimestamp(time as i64))
            .bind(user_username.into())
            .bind(new_username.into())
            .execute(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::Database(err))
                if err.is_unique_violation() && err.constraint() == Some("users_pkey") =>
            {
                return Err(DbUserUpdateUsernameErr::UsernameAlreadyUsed);
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbUserUpdateUsernameErr::UserNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserUpdateUsernameErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_username_change() {
    init_log();

    let db = Db::test_db(0, "test_user_username_change").await;

    let (user1, user2) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user1 = db
            .user_add(0, "hey", "hey", invite1.token.clone(), 10, 10)
            .await
            .unwrap();

        let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite2.token.clone(), 10, 10)
            .await
            .unwrap();

        (user1, user2)
    };

    // add data
    {
        use catsquad_shared::PostState;

        let post1 = db
            .post_add(0, user1.username.clone(), "title1", "", "")
            .await
            .unwrap();
        db.post_update_state(0, user1.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();
        db.post_like_add(0, user2.username.clone(), post1.id)
            .await
            .unwrap();
        db.password_change_add(0, user1.email.clone(), 10)
            .await
            .unwrap();
        db.invite_add(0, "hey3@heyadora.com", 10).await.unwrap();
        db.session_add(0, user1.email.clone()).await.unwrap();
        db.post_update_file_add(
            0,
            user1.username.clone(),
            post1.id,
            10,
            303043,
            "png",
            10,
            10,
        )
        .await
        .unwrap();
    }

    assert_eq!(user1.username, "hey");
    assert_eq!(user2.username, "hey2");

    let result = db
        .user_update_username(0, user1.username.clone(), "hey2")
        .await;

    assert!(matches!(
        result,
        Err(DbUserUpdateUsernameErr::UsernameAlreadyUsed)
    ));

    db.user_update_username(0, user1.username.clone(), "hey3")
        .await
        .unwrap();
    let user1 = db.user_get_by_email("hey@heyadora.com").await.unwrap();
    assert_eq!(user1.username, "hey3");

    db.user_update_username(0, user2.username.clone(), "hey4")
        .await
        .unwrap();
    let user2 = db.user_get_by_email("hey2@heyadora.com").await.unwrap();
    assert_eq!(user2.username, "hey4");
}
