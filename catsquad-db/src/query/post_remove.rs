use crate::Db;
use catsquad_log::prelude::*;
use catsquad_shared::PostState;

#[derive(Debug, thiserror::Error)]
pub enum DbPostRemoveErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("comment \"{0}\" was not found")]
    NotFound(i64),

    // #[error("user \"{0}\" was not found")]
    // UserNotFound(String),
    #[error("unauthorized")]
    Unauthorized,
}

impl Db {
    pub async fn post_remove(
        &self,
        user_username: impl Into<String>,
        post_id: i64,
    ) -> Result<(), DbPostRemoveErr> {
        let pool = &self.db;
        let user_username = user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get post
        {
            let query = "SELECT post_user_username, post_state FROM posts WHERE post_id = $1";

            let result = sqlx::query_as(query)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (post_user_username, post_state): (String, String) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbPostRemoveErr::NotFound(post_id));
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostRemoveErr::Db(err));
                }
            };

            if post_user_username != user_username {
                return Err(DbPostRemoveErr::Unauthorized);
            }

            let post_state = PostState::from(post_state);
            match post_state {
                PostState::Draft => return Err(DbPostRemoveErr::Unauthorized),
                PostState::Hidden => (),
                PostState::Active => (),
            }
        }

        // delete post like
        {
            let query = "DELETE FROM posts WHERE post_id = $1";

            let result = sqlx::query(query).bind(post_id).execute(&mut *tx).await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                // Err(sqlx::Error::RowNotFound) => {
                //     DOESNT GET TRIGGERED ON DELETE
                // }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostRemoveErr::Db(err));
                }
            };

            let affected = result.rows_affected();

            match affected {
                0 => return Err(DbPostRemoveErr::NotFound(post_id)),
                1 => (),
                _ => panic!("query is wrong, it must only delete 1 row max"),
            }
        }

        //

        tx.commit()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_remove() {
    // use crate::create_user_id;

    init_log();
    let db = Db::test_db(0, "test_post_remove").await;

    let (user, user2) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user = db
            .user_add(0, "hey", "hey", invite1.token, 10, 10)
            .await
            .unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite2.token, 10, 10)
            .await
            .unwrap();

        (user, user2)
    };

    let result = db.post_remove(user.username.clone(), 0).await;
    assert!(matches!(result, Err(DbPostRemoveErr::NotFound(_))));

    let post1_key = {
        let post1 = db
            .post_add(0, user.username.clone(), "title", "description", "tags")
            .await
            .unwrap();

        db.post_update_state(0, user.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();

        db.post_like_add(0, user2.username.clone(), post1.id)
            .await
            .unwrap();

        let comment1 = db
            .comment_add(0, user2.username.clone(), post1.id, None, "one")
            .await
            .unwrap();

        db.comment_add(
            0,
            user2.username.clone(),
            post1.id,
            Some(comment1.id),
            "one1",
        )
        .await
        .unwrap();

        post1.id
    };

    let post2_key = {
        let post2 = db
            .post_add(0, user.username.clone(), "title2", "description", "tags")
            .await
            .unwrap();
        db.post_update_state(0, user.username.clone(), post2.id, PostState::Active)
            .await
            .unwrap();

        db.post_like_add(0, user2.username.clone(), post2.id)
            .await
            .unwrap();

        db.comment_add(0, user.username.clone(), post2.id, None, "one2")
            .await
            .unwrap();
        post2.id
    };

    // assert errors
    {
        let result = db
            .post_remove(user2.username.clone(), post1_key.clone())
            .await;
        assert!(matches!(result, Err(DbPostRemoveErr::Unauthorized)));

        // let result = db.post_remove("invalid", post1_key.clone()).await;
        // assert!(matches!(result, Err(DbPostRemoveErr::UserNotFound(_))));

        let posts = db.post_get_all().await.unwrap();
        // let comments = db.comment_get_all().await.unwrap();
        let likes = db.post_like_get_all().await.unwrap();

        assert_eq!(posts.len(), 2);
        // assert_eq!(comments.len(), 3);
        assert_eq!(likes.len(), 2);
    }

    // assert success
    {
        db.post_remove(user.username.clone(), post1_key.clone())
            .await
            .unwrap();

        let posts = db.post_get_all().await.unwrap();
        let comments = db.comment_get_all().await.unwrap();
        let likes = db.post_like_get_all().await.unwrap();

        assert_eq!(posts.len(), 1);
        assert_eq!(comments.len(), 1);
        assert_eq!(likes.len(), 1);
        assert_eq!(posts[0].title, "title2");
        assert_eq!(comments[0].text, "one2");
        assert_eq!(likes[0].post_id, post2_key);
    }
}
