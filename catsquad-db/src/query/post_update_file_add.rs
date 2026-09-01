use crate::{Db, DbFileImage, DbImageKind, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPostUpdateFileAddErr {
    #[error("not enough storage")]
    OutOfStorage,

    #[error("file too big")]
    FileTooBig,

    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("file already exists")]
    FileAlreadyExists,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("something is terrbily wrnog: {0}")]
    InternalError(String),
}

// TODO make it so it doesnt use user's space if file is duplicate

impl Db {
    pub async fn post_update_file_add(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
        file_size: u32,
        file_hash: i64,
        file_extension: impl Into<String>,
        file_width: u32,
        file_height: u32,
    ) -> Result<DbFileImage, DbPostUpdateFileAddErr> {
        let user_username = user_username.into();
        let file_extension = file_extension.into();

        let mut tx = self.db.begin().await?;

        // get image
        let (image_exists, image_used_count, file_size) = {
            // let query = "SELECT EXISTS(SELECT 1 FROM files_images WHERE image_hash=$1)";
            let query =
                "SELECT image_used_count, image_size_bytes FROM files_images WHERE image_hash=$1";
            let result = sqlx::query_as(query)
                .bind(file_hash)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query}, result: {result:#?}");

            let result = match result {
                Ok(v) => {
                    let (image_usage_count, image_size_bytes): (i64, i64) = v;
                    (true, image_usage_count as u32 + 1, image_size_bytes as u32) // +1 because we will use this image next
                }
                Err(sqlx::Error::RowNotFound) => (false, 1, file_size), // 1 because we will create this img next
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateFileAddErr::Db(err));
                }
            };

            result
        };

        // get user
        let user_used_storage_bytes = {
            let query = "SELECT user_used_storage_bytes, user_max_storage_per_file_bytes, user_max_storage_bytes FROM users WHERE user_username = $1";

            let result = sqlx::query_as(query)
                .bind(&user_username)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query}, result: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error (post_update_file_add) {err}");
                    return Err(DbPostUpdateFileAddErr::Db(err));
                }
            };

            let (user_used_storage_bytes, user_max_storage_per_file_bytes, user_max_storage_bytes): (i64, i64, i64) = result;
            if user_used_storage_bytes < 0
                || user_max_storage_per_file_bytes < 0
                || user_max_storage_bytes < 0
            {
                let error = format!(
                    "negative storage detected in post_update_file_add user_used_storage_bytes({user_used_storage_bytes}) user_max_storage_per_file_bytes({user_max_storage_per_file_bytes}) user_max_storage_bytes({user_max_storage_bytes})"
                );
                error!("{error}");
                return Err(DbPostUpdateFileAddErr::InternalError(error));
            }

            let user_used_storage_bytes = user_used_storage_bytes as u32;
            let user_max_storage_per_file_bytes = user_max_storage_per_file_bytes as u32;
            let user_max_storage_bytes = user_max_storage_bytes as u32;

            if user_used_storage_bytes + file_size > user_max_storage_bytes {
                return Err(DbPostUpdateFileAddErr::OutOfStorage);
            }

            if file_size > user_max_storage_per_file_bytes {
                return Err(DbPostUpdateFileAddErr::FileTooBig);
            }

            user_used_storage_bytes
        };

        // get post
        let (post_images_hashes, post_size_bytes) = {
            let query = "SELECT post_user_username, post_images_hashes, post_size_bytes FROM posts WHERE post_id = $1";

            let result = sqlx::query_as(query)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbPostUpdateFileAddErr::PostNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateFileAddErr::Db(err));
                }
            };

            let (post_user_username, post_images_hashes, post_size_bytes): (String, Vec<i64>, i64) =
                result;

            if user_username != post_user_username {
                return Err(DbPostUpdateFileAddErr::Unauthorized);
            }

            let constains_hash = post_images_hashes.contains(&file_hash);
            if constains_hash {
                return Err(DbPostUpdateFileAddErr::FileAlreadyExists);
            }

            (post_images_hashes, post_size_bytes as u32)
        };

        if !image_exists {
            // insert image
            {
                let query = "
            INSERT INTO files_images (
                    image_hash,
                    image_extension,
                    image_kind,
                    image_size_bytes,
                    image_width,
                    image_height,
                    image_modified_at,
                    image_created_at
                )
                VALUES ( $1, $2, $3, $4, $5, $6, $7, $7 )
            ";

                let result = sqlx::query(query)
                    .bind(&file_hash)
                    .bind(&file_extension)
                    .bind(DbImageKind::Post.as_str())
                    .bind(file_size as i64)
                    .bind(file_width as i64)
                    .bind(file_height as i64)
                    .bind(XTimestamp(time as i64))
                    .execute(&mut *tx)
                    .await;

                debug!("query: {query}\nresult: {result:#?}");

                match result {
                    Ok(_v) => (),
                    Err(err) => {
                        error!("unexpected db error {err}");
                        return Err(DbPostUpdateFileAddErr::Db(err));
                    }
                };
            }
        } else {
            // image_used_count
            let query = "UPDATE files_images SET
                            image_used_count = image_used_count + 1,
                            image_modified_at = $2
                            WHERE image_hash = $1";

            let result = sqlx::query(query)
                .bind(file_hash)
                .bind(XTimestamp(time as i64))
                .execute(&mut *tx)
                .await;

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateFileAddErr::Db(err));
                }
            };

            debug!("query: {query}\nresult: {result:#?}");
        }

        // update post
        {
            let mut new_post_images_hashes = post_images_hashes;
            new_post_images_hashes.push(file_hash);
            let new_post_size_bytes = post_size_bytes + file_size;

            let query = "UPDATE posts SET
                            post_images_hashes = $1,
                            post_size_bytes = $4,
                            post_modified_at = $2
                            WHERE post_id = $3";

            let result = sqlx::query(query)
                .bind(&new_post_images_hashes)
                .bind(XTimestamp(time as i64))
                .bind(post_id)
                .bind(new_post_size_bytes as i64)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateFileAddErr::Db(err));
                }
            };

            let affected = result.rows_affected();
            if affected != 1 {
                tx.rollback().await.unwrap();
                panic!(
                    "something is wrong, updated wrong number of rows, expected 1, got {}, query {}, params {} {} {}",
                    affected, query, file_hash, time, post_id
                );
            }
        }

        // update user
        {
            let new_user_used_storage_bytes = user_used_storage_bytes + file_size;

            let query = "UPDATE users SET
                            user_used_storage_bytes = $1,
                            user_modified_at = $2
                            WHERE user_username = $3";

            let result = sqlx::query(query)
                .bind(new_user_used_storage_bytes as i64)
                .bind(XTimestamp(time as i64))
                .bind(&user_username)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostUpdateFileAddErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        let post_img = DbFileImage {
            hash: file_hash,
            extension: file_extension,
            size_bytes: file_size,
            used_count: image_used_count,
            kind: DbImageKind::Post.as_str().to_string(),
            processed: false,
            width: file_width,
            height: file_height,
            modified_at: time,
            created_at: time,
        };

        tx.commit().await?;

        Ok(post_img)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_file_add() {
    use catsquad_shared::PostState;
    init_log();

    let db = Db::test_db(0, "test_post_update_file_add").await;

    let (user1, user2) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user = db
            .user_add(0, "hey", "hey", invite1.token, 10, 5)
            .await
            .unwrap();

        let invite1 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite1.token, 10, 5)
            .await
            .unwrap();

        (user, user2)
    };

    let post1 = {
        let post1 = db
            .post_add(0, user1.username.clone(), "title1", "description1", "tags")
            .await
            .unwrap();
        assert_eq!(post1.images_hashes.len(), 0);

        db.post_update_state(0, user1.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();

        post1
    };

    // assert error
    {
        let result = db
            .post_update_file_add(0, user1.username.clone(), post1.id, 6, 10, "png", 10, 10)
            .await;
        assert!(matches!(result, Err(DbPostUpdateFileAddErr::FileTooBig)));
    }

    // assert success
    // users, posts and files_images tables must be updated
    {
        let post_img1 = db
            .post_update_file_add(0, user1.username.clone(), post1.id, 5, 10, "png", 10, 8)
            .await
            .unwrap();
        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id)
            .await
            .unwrap();
        let user1 = db
            .user_get_by_username(user1.username.clone())
            .await
            .unwrap();

        assert_eq!(post_img1.hash, 10);
        assert_eq!(post_img1.extension, "png");
        assert_eq!(post_img1.size_bytes, 5);
        assert_eq!(post_img1.kind, DbImageKind::Post.as_str());
        assert_eq!(post_img1.processed, false);
        assert_eq!(post_img1.used_count, 1);
        assert_eq!(post_img1.width, 10);
        assert_eq!(post_img1.height, 8);
        assert_eq!(post_img1.modified_at, 0);
        assert_eq!(post_img1.created_at, 0);

        assert_eq!(post1.size_bytes, 5);
        assert_eq!(post1.images_hashes.len(), 1);
        assert_eq!(post1.images_hashes[0], post_img1.hash);

        assert_eq!(user1.used_storage_bytes, 5);
    }

    // assert errors
    {
        let result = db
            .post_update_file_add(0, user1.username.clone(), post1.id, 50, 2, "png", 10, 10)
            .await;
        assert!(matches!(result, Err(DbPostUpdateFileAddErr::OutOfStorage)));

        let result = db
            .post_update_file_add(0, user1.username.clone(), post1.id, 3, 10, "png", 10, 10)
            .await;
        trace!("{result:?} == Err(DbPostUpdateFileAddErr::FileAlreadyExists)");
        assert!(matches!(
            result,
            Err(DbPostUpdateFileAddErr::FileAlreadyExists)
        ));

        let result = db
            .post_update_file_add(0, user1.username.clone(), 0, 1, 2, "png", 10, 10)
            .await;
        assert!(matches!(result, Err(DbPostUpdateFileAddErr::PostNotFound)));

        let result = db
            .post_update_file_add(0, user2.username.clone(), post1.id, 5, 2, "png", 10, 10)
            .await;
        assert!(matches!(result, Err(DbPostUpdateFileAddErr::Unauthorized)));
    }

    // assert success
    // second image, check if storage adds up correctly
    {
        let post_img2 = db
            .post_update_file_add(0, user1.username.clone(), post1.id, 2, 20, "png", 10, 8)
            .await
            .unwrap();
        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id)
            .await
            .unwrap();
        let user1 = db
            .user_get_by_username(user1.username.clone())
            .await
            .unwrap();

        assert_eq!(post_img2.hash, 20);
        assert_eq!(post_img2.size_bytes, 2);
        assert_eq!(post_img2.used_count, 1);

        assert_eq!(post1.size_bytes, 7);
        assert_eq!(post1.images_hashes.len(), 2);
        assert_eq!(post1.images_hashes[0], 10);
        assert_eq!(post1.images_hashes[1], 20);

        assert_eq!(user1.used_storage_bytes, 7);
    }

    let post2 = {
        let post = db
            .post_add(0, user1.username.clone(), "title2", "description2", "tags")
            .await
            .unwrap();
        assert_eq!(post.images_hashes.len(), 0);

        db.post_update_state(0, user1.username.clone(), post.id, PostState::Active)
            .await
            .unwrap();

        post
    };

    // assert success
    // second post
    {
        // 200 gets ignored, original file size is used
        let post_img2 = db
            .post_update_file_add(0, user1.username.clone(), post2.id, 200, 20, "png", 10, 8)
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
        let user1 = db
            .user_get_by_username(user1.username.clone())
            .await
            .unwrap();

        assert_eq!(post_img2.hash, 20);
        assert_eq!(post_img2.size_bytes, 2);
        assert_eq!(post_img2.used_count, 2);

        assert_eq!(post1.size_bytes, 7);
        assert_eq!(post1.images_hashes.len(), 2);
        assert_eq!(post1.images_hashes[0], 10);
        assert_eq!(post1.images_hashes[1], 20);

        assert_eq!(post2.size_bytes, 2);
        assert_eq!(post2.images_hashes.len(), 1);
        assert_eq!(post2.images_hashes[0], 20);

        assert_eq!(user1.used_storage_bytes, 9);
    }
}
