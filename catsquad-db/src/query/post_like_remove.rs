use crate::{Db, DbPostLike, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::PostState;

#[derive(Debug, thiserror::Error)]
pub enum DbPostLikeRemoveErr {
    #[error("post not found")]
    PostNotFound,

    #[error("like not found")]
    LikeNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_like_remove(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
    ) -> Result<(), DbPostLikeRemoveErr> {
        let pool = &self.db;

        let user_username = user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get post
        {
            let query = "SELECT post_state FROM posts WHERE post_id = $1";

            let result = sqlx::query_as(query)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (post_state,): (String,) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbPostLikeRemoveErr::PostNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostLikeRemoveErr::Db(err));
                }
            };

            let post_state = PostState::from(post_state);
            match post_state {
                PostState::Draft => return Err(DbPostLikeRemoveErr::Unauthorized),
                PostState::Hidden => return Err(DbPostLikeRemoveErr::Unauthorized),
                PostState::Active => (),
            }
        }

        // delete post like
        {
            let query = "DELETE FROM posts_likes WHERE post_like_user_username = $1 AND post_like_post_id = $2";

            let result = sqlx::query(query)
                .bind(user_username)
                .bind(post_id)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                // Err(sqlx::Error::RowNotFound) => {
                //     DOESNT GET TRIGGERED ON EXECUTE
                // }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostLikeRemoveErr::Db(err));
                }
            };

            let affected = result.rows_affected();

            match affected {
                0 => return Err(DbPostLikeRemoveErr::LikeNotFound),
                1 => (),
                _ => panic!("query is wrong, it must only delete 1 row max"),
            }
        }

        // update post total like count
        {
            let query = "UPDATE posts SET
                            post_likes_count = post_likes_count - 1,
                            post_modified_at = $1
                            WHERE post_id = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(post_id)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostLikeRemoveErr::Db(err));
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
async fn test_post_like_remove() {
    init_log();
    let db = Db::test_db(0, "test_post_like_remove").await;

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

    let post1 = db
        .post_add(0, user1.username.clone(), "title", "description", "tags")
        .await
        .unwrap();

    // assert errors
    {
        let result = db.post_like_remove(0, user1.username.clone(), 0).await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::PostNotFound)));

        let result = db
            .post_like_remove(0, user1.username.clone(), post1.id)
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::Unauthorized)));

        let result = db
            .post_like_remove(0, user2.username.clone(), post1.id)
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::Unauthorized)));
    }

    db.post_update_state(
        0,
        user1.username.clone(),
        post1.id,
        catsquad_shared::PostState::Active,
    )
    .await
    .unwrap();

    let post2 = {
        let post2 = db
            .post_add(0, user1.username.clone(), "title2", "description", "tags")
            .await
            .unwrap();
        db.post_update_state(
            0,
            user1.username.clone(),
            post2.id,
            catsquad_shared::PostState::Active,
        )
        .await
        .unwrap();
        post2
    };

    // assert errors
    {
        let result = db
            .post_like_remove(0, user1.username.clone(), post1.id)
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::LikeNotFound)));

        let result = db
            .post_like_remove(0, user2.username.clone(), post1.id)
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::LikeNotFound)));
    }

    // assert success
    {
        db.post_like_add(0, user2.username.clone(), post1.id)
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id)
            .await
            .unwrap();
        let post2 = db
            .post_get_by_id(user1.username.clone(), post2.id)
            .await
            .unwrap();
        assert_eq!(post1.likes_count, 1);
        assert_eq!(post2.likes_count, 0);

        db.post_like_remove(0, user2.username.clone(), post1.id)
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id)
            .await
            .unwrap();
        let post2 = db
            .post_get_by_id(user1.username.clone(), post2.id)
            .await
            .unwrap();
        assert_eq!(post1.likes_count, 0);
        assert_eq!(post2.likes_count, 0);
    }

    db.post_update_state(
        0,
        user1.username.clone(),
        post1.id,
        catsquad_shared::PostState::Hidden,
    )
    .await
    .unwrap();

    // assert errors
    {
        let result = db
            .post_like_remove(0, user1.username.clone(), post1.id)
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::Unauthorized)));

        let result = db
            .post_like_remove(0, user2.username.clone(), post1.id)
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::Unauthorized)));
    }
}
