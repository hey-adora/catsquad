use crate::Db;
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPostLikeExistsByPostErr {
    #[error("not found")]
    NotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_like_exists_by_post(
        &self,
        user_username: impl Into<String>,
        post_id: i64,
    ) -> Result<(), DbPostLikeExistsByPostErr> {
        let pool = &self.db;

        let user_username = user_username.into();

        let query = "SELECT EXISTS(SELECT 1 FROM posts_likes WHERE post_like_user_username=$1 AND post_like_post_id=$2)";

        let result = sqlx::query_as(query)
            .bind(user_username)
            .bind(post_id)
            .fetch_one(pool)
            .await;

        debug!("query: {query}\nresult: {result:#?}");

        match result {
            Ok((true,)) => Ok(()),
            Ok((false,)) => Err(DbPostLikeExistsByPostErr::NotFound),
            Err(err) => {
                error!("unexpected db error {err}");
                Err(DbPostLikeExistsByPostErr::Db(err))
            }
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_like_exists_by_post() {
    init_log();
    let db = Db::test_db(0, "test_post_like_exists_by_post").await;

    let (user1, user2) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user1 = db
            .user_add(0, "hey", "hey", invite1.token, 10, 10)
            .await
            .unwrap();

        let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite2.token, 10, 10)
            .await
            .unwrap();

        (user1, user2)
    };

    let post1 = {
        let post1 = db
            .post_add(0, user1.username.clone(), "title", "description", "tags")
            .await
            .unwrap();
        db.post_update_state(
            0,
            user1.username.clone(),
            post1.id,
            catsquad_shared::PostState::Active,
        )
        .await
        .unwrap();
        post1
    };

    // assert errors
    {
        let result = db
            .post_like_exists_by_post(user2.username.clone(), post1.id)
            .await;
        assert!(matches!(result, Err(DbPostLikeExistsByPostErr::NotFound)));
    }

    // assert success
    {
        db.post_like_add(0, user2.username.clone(), post1.id)
            .await
            .unwrap();

        db.post_like_exists_by_post(user2.username.clone(), post1.id)
            .await
            .unwrap();
    }
}
