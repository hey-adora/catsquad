use crate::{Db, query::post_like_add::DbPostLike};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPostLikeGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_like_get_all(&self) -> Result<Vec<DbPostLike>, DbPostLikeGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM posts_likes ORDER BY post_like_created_at DESC";

        let result = sqlx::query_as(query).fetch_all(pool).await;

        debug!("query {query} result {result:#?}");

        let post_likes = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostLikeGetAllErr::Db(err));
            }
        };

        Ok(post_likes)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_like_get_all() {
    init_log();

    let db = Db::test_db(0, "test_post_like_get_all").await;

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

    let _post_like = db
        .post_like_add(0, user2.username.clone(), post1.id)
        .await
        .unwrap();

    let post_likes = db.post_like_get_all().await.unwrap();
    assert_eq!(post_likes.len(), 1);
}
