use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::i64_to_str;

#[derive(Debug, thiserror::Error)]
pub enum DbFileImageRemoveErr {
    #[error("file already exists")]
    FileNotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal error {0}")]
    InternalError(String),
}

impl Db {
    pub async fn file_image_remove(
        &self,
        time: u64,
        file_hash: i64,
    ) -> Result<(), DbFileImageRemoveErr> {
        let mut tx = self.db.begin().await?;

        // delete image
        let (image_size_bytes, image_used_count) = {
            let query = "DELETE FROM files_images
                                 WHERE image_hash = $1
                                 RETURNING image_size_bytes, image_used_count";

            let result = sqlx::query_as(query)
                .bind(file_hash)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query}, result: {result:#?}");

            let result: (i64, i64) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbFileImageRemoveErr::FileNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFileImageRemoveErr::Db(err));
                }
            };

            result
        };

        if image_used_count < 0 {
            return Err(DbFileImageRemoveErr::InternalError("image_used_count is less than 0 = {image_used_count} for file_image {file_hash}. it should have been deleted when it was at 0".to_string()));
        }

        // update posts
        let usernames_and_sizes: Vec<(String, i64)> = {
            let query = "UPDATE posts SET
                                    post_images_hashes = array_remove(post_images_hashes, $1),
                                    post_size_bytes = post_size_bytes - $3,
                                    post_modified_at = $2
                                    WHERE $1 = ANY(post_images_hashes)
                                    RETURNING post_user_username

                                    ";

            let result = sqlx::query_as(query)
                .bind(file_hash)
                .bind(XTimestamp(time as i64))
                .bind(image_size_bytes)
                .fetch_all(&mut *tx)
                .await;

            debug!(
                "query {query}, $1={file_hash}, $2={time}, $3={image_size_bytes}, result: {result:#?}"
            );

            let result: Vec<(String,)> = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFileImageRemoveErr::Db(err));
                }
            };

            if result.len() as i64 != image_used_count {
                error!(
                    "image magically dissapeared from posts without updating image_used_count hash={file_hash} hash_str={}",
                    i64_to_str(file_hash)
                );
            }

            calc_user_used_storage(result, image_size_bytes)
        };

        // update user
        {
            let query = "UPDATE users SET
                            user_used_storage_bytes = user_used_storage_bytes - $1,
                            user_modified_at = $2
                            WHERE user_username = $3";

            for (username, size) in usernames_and_sizes {
                let result = sqlx::query(query)
                    .bind(size)
                    .bind(XTimestamp(time as i64))
                    .bind(&username)
                    .execute(&mut *tx)
                    .await;

                debug!(
                    "query {query}, $1={image_size_bytes} $2={time} $3={username:?}, result: {result:#?}"
                );

                let _result = match result {
                    Ok(v) => v,
                    Err(err) => {
                        error!("unexpected db error {err}");
                        return Err(DbFileImageRemoveErr::Db(err));
                    }
                };
            }
        }

        tx.commit().await?;

        Ok(())
        // Ok(img_used_count)
    }
}

fn calc_user_used_storage(mut usernames: Vec<(String,)>, file_size: i64) -> Vec<(String, i64)> {
    usernames.sort();

    let mut usernames_and_sizes = Vec::new();

    let mut prev_item = String::new();
    let mut index = 0_usize;
    for username in usernames.into_iter().map(|v| v.0) {
        if prev_item != username {
            prev_item = username.clone();
            usernames_and_sizes.push((username, file_size));
            index += 1;
            continue;
        }
        usernames_and_sizes[index - 1].1 += file_size;
    }

    usernames_and_sizes
}

#[cfg(test)]
#[tokio::test]
async fn test_calc_user_used_storage() {
    let result = calc_user_used_storage(
        vec![
            ("hey".to_string(),),
            ("hey".to_string(),),
            ("hey3".to_string(),),
        ],
        10,
    );
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "hey");
    assert_eq!(result[0].1, 20);
    assert_eq!(result[1].0, "hey3");
    assert_eq!(result[1].1, 10);
}

#[cfg(test)]
#[tokio::test]
async fn test_file_image_remove() {
    init_log();

    let db = Db::test_db(0, "test_file_image_remove").await;

    let user = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user = db
            .user_add(0, "hey", "hey", invite1.token, 35, 15)
            .await
            .unwrap();
        user
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
    }

    // remove
    {
        db.file_image_remove(0, 12).await.unwrap();

        let post1 = db
            .post_get_by_id(user.username.clone(), post1.id)
            .await
            .unwrap();

        let post2 = db
            .post_get_by_id(user.username.clone(), post2.id)
            .await
            .unwrap();

        let file_img1 = db.file_image_get_by_hash(12).await;

        let user1 = db
            .user_get_by_username(user.username.clone())
            .await
            .unwrap();

        assert!(matches!(
            file_img1,
            Err(crate::DbFileImageGetByHashErr::NotFound)
        ));

        assert_eq!(post1.size_bytes, 5);
        assert_eq!(post1.images.len(), 1);
        assert_eq!(post1.images[0].hash, 11);

        assert_eq!(post2.size_bytes, 0);
        assert_eq!(post2.images.len(), 0);

        assert_eq!(user1.used_storage_bytes, 5);
    }
}
