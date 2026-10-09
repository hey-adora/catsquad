use crate::{Db, XTimestamp, query::file_image_remove_tx::DbFileImageRemoveTxErr};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPostUpdateImageRemoveErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("file already exists")]
    FileNotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal error {0}")]
    InternalError(String),

    #[error("image remove error {0}")]
    ImageRemove(#[from] DbFileImageRemoveTxErr),
}

impl Db {
    pub async fn post_update_file_remove(
        &self,
        time: u64,
        callback_remove_image: impl AsyncFnOnce(i64, &str),
        user_username: impl Into<String>,
        post_id: i64,
        file_hash: i64,
    ) -> Result<u32, DbPostUpdateImageRemoveErr> {
        let user_username = user_username.into();
        let mut tx = self
            .db
            .begin()
            .await
            .inspect_err(|err| error!("post_update_file_remove {err}"))?;

        // get post
        let (post_img_pos, post_images_hashes, post_size_bytes) = {
            let query = "SELECT post_user_username, post_images_hashes, post_size_bytes FROM posts WHERE post_id = $1";
            // let query = "SELECT image_used_count FROM files_images WHERE image_hash=$1";
            let result = sqlx::query_as(query)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query}, result: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbPostUpdateImageRemoveErr::PostNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateImageRemoveErr::Db(err));
                }
            };
            let (post_user_username, post_images_hashes, post_size_bytes): (String, Vec<i64>, i64) =
                result;

            if post_user_username != user_username {
                return Err(DbPostUpdateImageRemoveErr::Unauthorized);
            }

            let Some(post_img_pos) = post_images_hashes.iter().position(|v| *v == file_hash) else {
                return Err(DbPostUpdateImageRemoveErr::FileNotFound);
            };

            (post_img_pos, post_images_hashes, post_size_bytes as u32)
        };

        // remove image
        let (img_used_count, img_size_bytes, extension) =
            self.file_image_remove_tx(&mut *tx, time, file_hash).await?;

        // update post
        {
            let new_post_size_bytes = post_size_bytes.checked_sub(img_size_bytes).ok_or_else(|| DbPostUpdateImageRemoveErr::InternalError(format!("{post_size_bytes} - {img_size_bytes} < 0 = post_size_bytes - img_size_bytes = post_id {post_id}"))).inspect_err(|err| error!("{err}"))?;
            let mut new_post_images_hashes = post_images_hashes;
            new_post_images_hashes.remove(post_img_pos);

            let query = "UPDATE posts SET
                            post_images_hashes = $1,
                            post_size_bytes = $2,
                            post_modified_at = $3
                            WHERE post_id = $4";

            let result = sqlx::query(query)
                .bind(new_post_images_hashes)
                .bind(new_post_size_bytes as i64)
                .bind(XTimestamp(time as i64))
                .bind(post_id)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateImageRemoveErr::Db(err));
                }
            };

            if result.rows_affected() != 1 {
                return Err(DbPostUpdateImageRemoveErr::InternalError(
                    "rows affected != 1".to_string(),
                ));
            }
        }

        // update user
        {
            // let new_user_used_storage_bytes = user_used_storage_bytes + file_size;

            let query = "UPDATE users SET
                            user_used_storage_bytes = user_used_storage_bytes - $1,
                            user_modified_at = $2
                            WHERE user_username = $3";

            let result = sqlx::query(query)
                .bind(img_size_bytes as i64)
                .bind(XTimestamp(time as i64))
                .bind(&user_username)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateImageRemoveErr::Db(err));
                }
            };

            if result.rows_affected() != 1 {
                return Err(DbPostUpdateImageRemoveErr::InternalError(
                    "rows affected != 1".to_string(),
                ));
            }
        }

        if img_used_count == 0 {
            callback_remove_image(file_hash, extension.as_str()).await;
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("post_update_file_remove {err}"))?;

        Ok(img_used_count)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_file_remove() {
    init_log();

    let db = Db::test_db(0, "test_post_update_file_remove").await;

    let callback_remove_file = async move |hash: i64, extension: &str| {
        //
    };

    let (user, user2) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user = db
            .user_add(0, "hey", "hey", invite1.token, 35, 15)
            .await
            .unwrap();

        let invite1 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite1.token, 10, 10)
            .await
            .unwrap();

        (user, user2)
    };

    let (post1, post2) = {
        use catsquad_shared::PostState;

        let post1 = db
            .post_add(0, user.username.clone(), "title1", "description1", "tags")
            .await
            .unwrap();
        db.post_update_state(0, user.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();

        let post2 = db
            .post_add(0, user.username.clone(), "title2", "description2", "tags")
            .await
            .unwrap();
        db.post_update_state(0, user.username.clone(), post2.id, PostState::Active)
            .await
            .unwrap();

        (post1, post2)
    };

    // add few imgs
    {
        db.post_update_image_add(0, user.username.clone(), post2.id, 15, 12, "png", 10, 10)
            .await
            .unwrap();
        db.post_update_image_add(0, user.username.clone(), post1.id, 5, 11, "png", 10, 10)
            .await
            .unwrap();
        db.post_update_image_add(0, user.username.clone(), post1.id, 15, 12, "png", 10, 10)
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user.username.clone(), post1.id)
            .await
            .unwrap();
        let user1 = db
            .user_get_by_username(user.username.clone())
            .await
            .unwrap();
        let img1 = db.file_image_get_by_hash(11).await.unwrap();
        let img2 = db.file_image_get_by_hash(12).await.unwrap();

        assert_eq!(post1.images.len(), 2);
        assert_eq!(post1.images[0].hash, 11);
        assert_eq!(post1.images[1].hash, 12);
        assert_eq!(post1.size_bytes, 20);
        assert_eq!(user1.used_storage_bytes, 35);
        assert_eq!(img1.used_count, 1);
        assert_eq!(img2.used_count, 2);
    }

    // assert errors
    {
        let result = db
            .post_update_file_remove(0, callback_remove_file, user.username.clone(), 0, 0)
            .await;
        assert!(matches!(
            result,
            Err(DbPostUpdateImageRemoveErr::PostNotFound)
        ));

        let result = db
            .post_update_file_remove(0, callback_remove_file, user2.username.clone(), post1.id, 0)
            .await;
        assert!(matches!(
            result,
            Err(DbPostUpdateImageRemoveErr::Unauthorized)
        ));

        let result = db
            .post_update_file_remove(0, callback_remove_file, user.username.clone(), post1.id, 0)
            .await;
        assert!(matches!(
            result,
            Err(DbPostUpdateImageRemoveErr::FileNotFound)
        ));
    }

    // assert success
    {
        use crate::DbFileImageGetByHashErr;

        let post1_file = db
            .post_update_file_remove(0, callback_remove_file, user.username.clone(), post1.id, 11)
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user.username.clone(), post1.id)
            .await
            .unwrap();
        let user1 = db
            .user_get_by_username(user.username.clone())
            .await
            .unwrap();
        let img1 = db.file_image_get_by_hash(11).await;
        let img2 = db.file_image_get_by_hash(12).await.unwrap();

        assert_eq!(post1_file, 0);
        assert_eq!(post1.images.len(), 1);
        assert_eq!(post1.images[0].hash, 12);
        assert_eq!(post1.size_bytes, 15);
        assert_eq!(user1.used_storage_bytes, 30);
        assert!(matches!(img1, Err(DbFileImageGetByHashErr::NotFound)));
        assert_eq!(img2.used_count, 2);
    }

    // assert success
    {
        use crate::DbFileImageGetByHashErr;

        let post1_file = db
            .post_update_file_remove(0, callback_remove_file, user.username.clone(), post1.id, 12)
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user.username.clone(), post1.id)
            .await
            .unwrap();
        let user1 = db
            .user_get_by_username(user.username.clone())
            .await
            .unwrap();
        let img1 = db.file_image_get_by_hash(11).await;
        let img2 = db.file_image_get_by_hash(12).await.unwrap();

        assert_eq!(post1_file, 1);
        assert_eq!(post1.images.len(), 0);
        assert_eq!(post1.size_bytes, 0);
        assert_eq!(user1.used_storage_bytes, 15);
        assert!(matches!(img1, Err(DbFileImageGetByHashErr::NotFound)));
        assert_eq!(img2.used_count, 1);
    }
}
