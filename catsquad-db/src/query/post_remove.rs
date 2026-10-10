use crate::{Db, DbFileImageRemoveTxErr, XTimestamp};
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

    #[error(transparent)]
    Image(#[from] DbFileImageRemoveTxErr),
}

impl Db {
    pub async fn post_remove(
        &self,
        time: u64,
        mut callback_remove_image: impl AsyncFnMut(i64, &str),
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
        let images = {
            let query = "SELECT post_user_username, post_state, post_images_hashes FROM posts WHERE post_id = $1";

            let result = sqlx::query_as(query)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (post_user_username, post_state, images): (String, String, Vec<i64>) = match result
            {
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

            images
        };

        let mut total_removed_imgs_size = 0;
        for hash in images {
            let (img_used_count, img_size, extension) =
                self.file_image_remove_tx(&mut *tx, time, hash).await?;
            if img_used_count == 0 {
                callback_remove_image(hash, extension.as_str()).await;
            }
            total_removed_imgs_size += img_size;
        }

        // update user
        {
            let query = "UPDATE users SET
                            user_used_storage_bytes = user_used_storage_bytes - $1,
                            user_modified_at = $2
                            WHERE user_username = $3";

            let result = sqlx::query(query)
                .bind(total_removed_imgs_size as i64)
                .bind(XTimestamp(time as i64))
                .bind(&user_username)
                .execute(&mut *tx)
                .await;

            debug!(
                "query {query}, $1={total_removed_imgs_size } $2={time} $3={user_username:?}, result: {result:#?}"
            );

            let _result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostRemoveErr::Db(err));
                }
            };
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

    let callback_remove = async move |hash: i64, extension: &str| {
        //
    };

    let result = db
        .post_remove(0, callback_remove, user.username.clone(), 0)
        .await;
    assert!(matches!(result, Err(DbPostRemoveErr::NotFound(_))));

    let post1_key = {
        let post1 = db
            .post_add(0, user.username.clone(), "title", "description", "tags")
            .await
            .unwrap();

        db.post_update_state(0, user.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();

        db.post_update_image_add(0, user.username.clone(), post1.id, 1, 666, "png", 15, 15)
            .await
            .unwrap();

        db.post_update_image_add(0, user.username.clone(), post1.id, 1, 667, "png", 15, 15)
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

        db.post_update_image_add(0, user.username.clone(), post2.id, 1, 667, "png", 15, 15)
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
            .post_remove(
                0,
                callback_remove,
                user2.username.clone(),
                post1_key.clone(),
            )
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
        use crate::DbFileImageGetByHashErr;

        let user1 = db.user_get_by_username("hey").await.unwrap();
        let user2 = db.user_get_by_username("hey2").await.unwrap();
        let img1 = db.file_image_get_by_hash(666).await.unwrap();
        let img2 = db.file_image_get_by_hash(667).await.unwrap();

        assert_eq!(user1.used_storage_bytes, 3);
        assert_eq!(user2.used_storage_bytes, 0);
        assert_eq!(img1.used_count, 1);
        assert_eq!(img2.used_count, 2);

        db.post_remove(0, callback_remove, user.username.clone(), post1_key.clone())
            .await
            .unwrap();

        let user1 = db.user_get_by_username("hey").await.unwrap();
        let user2 = db.user_get_by_username("hey2").await.unwrap();
        let img1 = db.file_image_get_by_hash(666).await;
        let img2 = db.file_image_get_by_hash(667).await.unwrap();

        let posts = db.post_get_all().await.unwrap();
        let comments = db.comment_get_all().await.unwrap();
        let likes = db.post_like_get_all().await.unwrap();

        assert_eq!(user1.used_storage_bytes, 1);
        assert_eq!(user2.used_storage_bytes, 0);
        assert!(matches!(img1, Err(DbFileImageGetByHashErr::NotFound)));
        assert_eq!(img2.used_count, 1);

        assert_eq!(posts.len(), 1);
        assert_eq!(comments.len(), 1);
        assert_eq!(likes.len(), 1);
        assert_eq!(posts[0].title, "title2");
        assert_eq!(comments[0].text, "one2");
        assert_eq!(likes[0].post_id, post2_key);
    }
}
