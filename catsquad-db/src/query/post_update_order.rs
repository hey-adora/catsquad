use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPostUpdateOrderErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("invalid selected index")]
    InvalidIndex,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal error {0}")]
    InternalError(String),
}

impl Db {
    pub async fn post_update_order(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
        selected_pos: usize,
        new_pos: usize,
    ) -> Result<(), DbPostUpdateOrderErr> {
        let user_username = user_username.into();

        let mut tx = self.db.begin().await?;

        // get post
        let post_images_hashes = {
            let query =
                "SELECT post_user_username, post_images_hashes FROM posts WHERE post_id = $1";

            let result = sqlx::query_as(query)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbPostUpdateOrderErr::PostNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateOrderErr::Db(err));
                }
            };

            let (post_user_username, post_images_hashes): (String, Vec<i64>) = result;
            let hashes_len = post_images_hashes.len();

            if post_user_username != user_username {
                return Err(DbPostUpdateOrderErr::Unauthorized);
            }

            if new_pos >= hashes_len || selected_pos >= hashes_len {
                return Err(DbPostUpdateOrderErr::InvalidIndex);
            }

            post_images_hashes
        };

        // update post
        {
            let mut new_post_images_hashes = post_images_hashes;
            let hash = new_post_images_hashes.remove(selected_pos);
            new_post_images_hashes.insert(new_pos, hash);
            // new_post_images_hashes.swap(selected_pos, new_pos);

            let query = "UPDATE posts SET
                            post_images_hashes = $1,
                            post_modified_at = $2
                            WHERE post_id = $3";

            let result = sqlx::query(query)
                .bind(&new_post_images_hashes)
                .bind(XTimestamp(time as i64))
                .bind(post_id)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateOrderErr::Db(err));
                }
            };

            let affected = result.rows_affected();
            if result.rows_affected() != 1 {
                return Err(DbPostUpdateOrderErr::InternalError(format!(
                    "post_update_order must updated 1 row max, updated {affected}"
                )));
            }
        }

        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_order() {
    use crate::DbPost;

    init_log();

    let db = Db::test_db(0, "test_post_update_order").await;

    let (user, user2) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user = db
            .user_add(0, "hey", "hey", invite1.token, 10, 10)
            .await
            .unwrap();

        let invite1 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite1.token, 10, 10)
            .await
            .unwrap();

        (user, user2)
    };

    let post = {
        use catsquad_shared::PostState;

        let post = db
            .post_add(0, user.username.clone(), "title", "description", "")
            .await
            .unwrap();
        db.post_update_state(0, user.username.clone(), post.id, PostState::Active)
            .await
            .unwrap();

        post
    };

    // add images
    {
        let add_post_file_fn = async |post: &DbPost, hash: i64, size: u32| {
            db.post_update_file_add(0, user.username.clone(), post.id, size, hash, "png", 50, 50)
                .await
        };

        add_post_file_fn(&post, 10, 1).await.unwrap();
        add_post_file_fn(&post, 20, 1).await.unwrap();
        add_post_file_fn(&post, 30, 1).await.unwrap();
        add_post_file_fn(&post, 40, 1).await.unwrap();
        let post = db
            .post_get_by_id(user.username.clone(), post.id)
            .await
            .unwrap();

        assert_eq!(post.images.len(), 4);
        assert_eq!(post.images[0].hash, 10);
        assert_eq!(post.images[1].hash, 20);
        assert_eq!(post.images[2].hash, 30);
        assert_eq!(post.images[3].hash, 40);
    }

    // assert success
    {
        db.post_update_order(0, user.username.clone(), post.id, 2, 0)
            .await
            .unwrap();
        let post = db
            .post_get_by_id(user.username.clone(), post.id)
            .await
            .unwrap();
        assert_eq!(post.images.len(), 4);
        assert_eq!(post.images[0].hash, 30);
        assert_eq!(post.images[1].hash, 10);
        assert_eq!(post.images[2].hash, 20);
        assert_eq!(post.images[3].hash, 40);

        db.post_update_order(0, user.username.clone(), post.id, 0, 2)
            .await
            .unwrap();
        let post = db
            .post_get_by_id(user.username.clone(), post.id)
            .await
            .unwrap();
        assert_eq!(post.images.len(), 4);
        assert_eq!(post.images[0].hash, 10);
        assert_eq!(post.images[1].hash, 20);
        assert_eq!(post.images[2].hash, 30);
        assert_eq!(post.images[3].hash, 40);

        db.post_update_order(0, user.username.clone(), post.id, 0, 3)
            .await
            .unwrap();
        let post = db
            .post_get_by_id(user.username.clone(), post.id)
            .await
            .unwrap();
        assert_eq!(post.images.len(), 4);
        assert_eq!(post.images[0].hash, 20);
        assert_eq!(post.images[1].hash, 30);
        assert_eq!(post.images[2].hash, 40);
        assert_eq!(post.images[3].hash, 10);
    }

    // assert errors
    {
        let result = db
            .post_update_order(0, user2.username.clone(), post.id, 2, 0)
            .await;
        assert!(matches!(result, Err(DbPostUpdateOrderErr::Unauthorized)));

        let post_err = db
            .post_update_order(0, user.username.clone(), post.id, 0, 4)
            .await
            .err()
            .unwrap();
        assert!(matches!(post_err, DbPostUpdateOrderErr::InvalidIndex));

        let post_err = db
            .post_update_order(0, user.username.clone(), post.id, 4, 0)
            .await
            .err()
            .unwrap();
        assert!(matches!(post_err, DbPostUpdateOrderErr::InvalidIndex));

        let post_err = db
            .post_update_order(0, user.username.clone(), 0, 4, 0)
            .await
            .err()
            .unwrap();
        assert!(matches!(post_err, DbPostUpdateOrderErr::PostNotFound));
    }
}
