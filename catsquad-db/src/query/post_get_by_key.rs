use crate::{Db, DbInvite, DbPost};
use catsquad_log::prelude::*;
use catsquad_shared::{POST_STATE_DRAFT, POST_STATE_HIDDEN, PostState};

#[derive(Debug, thiserror::Error)]
pub enum DbPostGetByKeyErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_get_by_id(
        &self,
        user_username: String,
        post_key: i64,
    ) -> Result<DbPost, DbPostGetByKeyErr> {
        let pool = &self.db;

        let query = "SELECT * FROM posts WHERE post_id = $1 AND post_state != 'draft'";

        debug!("about to run {query}");

        let result = sqlx::query_as(query).bind(post_key).fetch_one(pool).await;

        let post: DbPost = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbPostGetByKeyErr::PostNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostGetByKeyErr::Db(err));
            }
        };

        if post.state == POST_STATE_HIDDEN && post.user_username != user_username {
            return Err(DbPostGetByKeyErr::Unauthorized);
        }

        debug!("query {query}\nresults {post:?}");

        Ok(post)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_get_by_key() {
    init_log();

    let db = Db::test_db(0, "test_post_get_by_key").await;

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

    let post1 = db
        .post_add(0, user1.username.clone(), "title1", "description1", "tags")
        .await
        .unwrap();

    {
        let result = db
            .post_get_by_id(user1.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::PostNotFound)));

        let result = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::PostNotFound)));
    }

    db.post_update_state(
        0,
        user1.username.clone(),
        post1.id.clone(),
        PostState::Active,
    )
    .await
    .unwrap();

    {
        let result = db
            .post_get_by_id(user1.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));

        let result = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));
    }

    db.post_update_state(
        0,
        user1.username.clone(),
        post1.id.clone(),
        PostState::Hidden,
    )
    .await
    .unwrap();

    {
        let result = db
            .post_get_by_id(user1.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));

        let result = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized)));
    }
}
