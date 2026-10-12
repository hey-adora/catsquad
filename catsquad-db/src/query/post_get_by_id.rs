use crate::{Db, DbFileImage, XTimestamp, run_if_not_empty};
use catsquad_log::prelude::*;
use catsquad_shared::POST_STATE_HIDDEN;
use sqlx::{AssertSqlSafe, FromRow};

#[derive(Debug, thiserror::Error)]
pub enum DbPostGetByKeyErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbPostGet {
    #[sqlx(rename = "post_id")]
    pub id: i64,
    #[sqlx(rename = "post_user_username")]
    pub user_username: String,
    #[sqlx(rename = "post_user_pfp_hash")]
    pub user_pfp_hash: i64,
    #[sqlx(rename = "post_user_pfp_proccesed")]
    pub user_pfp_proccesed: bool,
    #[sqlx(rename = "post_state")]
    pub state: String, // TODO optimize to be enum
    #[sqlx(rename = "post_title")]
    pub title: String,
    #[sqlx(rename = "post_description")]
    pub description: String,
    #[sqlx(rename = "post_tags")]
    pub tags: String,
    #[sqlx(rename = "post_liked")]
    pub liked: bool,
    #[sqlx(rename = "post_likes_count")]
    #[sqlx(try_from = "i64")]
    pub likes_count: u32,
    #[sqlx(rename = "post_size_bytes")]
    #[sqlx(try_from = "i64")]
    pub size_bytes: u32,
    #[sqlx(skip)]
    pub images: Vec<DbFileImage>,
    #[sqlx(rename = "post_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "post_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

// impl From<DbPost> for DbPostGet {
//     fn from(value: DbPost) -> Self {
//         Self {
//             id: value.id,
//             user_username: value.user_username,
//             user_pfp_hash: value.p,
//             state: value.state,
//             title: value.title,
//             description: value.description,
//             tags: value.tags,
//             likes_count: value.likes_count,
//             size_bytes: value.size_bytes,
//             images: Vec::new(),
//             modified_at: value.modified_at,
//             created_at: value.created_at,
//         }
//     }
// }

impl Db {
    pub async fn post_get_by_id(
        &self,
        user_username: String,
        post_id: i64,
    ) -> Result<DbPostGet, DbPostGetByKeyErr> {
        let pool = &self.db;
        // coalesce(l2.post_like_post_id > 0, false) as post_liked
        // coalesce(f2.image_processed, false) as post_user_pfp_proccesed,
        // coalesce(f1.image_width, 0) as post_image_width,
        // coalesce(f1.image_height, 0) as post_image_height,
        // coalesce(f1.image_extension, '') as post_image_extension,
        // coalesce(f1.image_hash, 0) as post_image_hash,
        //user_pfp_image_hash
        // false as post_liked,
        // user_pfp_image_hash as post_user_pfp_hash,
        // coalesce(f1.image_processed, false) as post_user_pfp_proccesed,
        // JOIN users ON post_user_username=user_username
        // LEFT JOIN files_images f1 ON user_pfp_image_hash=f1.image_hash

        // CAST(0 as int8) as post_user_pfp_hash,
        // false as post_user_pfp_proccesed,

        let q_liked_by_me = 'a: {
            if !user_username.is_empty() {
                break 'a "LEFT JOIN posts_likes ON post_id=post_like_post_id AND post_like_user_username = $2";
            }

            ""
        };

        let q_liked_by_me_field = 'a: {
            if !user_username.is_empty() {
                break 'a "coalesce(post_like_post_id > 0, false) as post_liked";
            }

            "false as post_liked"
        };

        // get post
        let (images_hashes, post) = {
            let query_str = format!(
                "
                            SELECT
                                    post_images_hashes,

                                    user_pfp_image_hash as post_user_pfp_hash,
                                    coalesce(image_processed, false) as post_user_pfp_proccesed,

                                    {q_liked_by_me_field},

                                    post_id,
                                    post_user_username,
                                    post_state,
                                    post_title,
                                    post_description,
                                    post_tags,
                                    post_likes_count,
                                    post_size_bytes,
                                    post_modified_at,
                                    post_created_at
                                FROM posts

                                JOIN users ON post_user_username=user_username
                                LEFT JOIN files_images ON user_pfp_image_hash=image_hash

                                {q_liked_by_me}

                                WHERE post_id = $1 AND post_state != 'draft'"
            );

            let query = AssertSqlSafe(query_str.as_str());
            let builder = sqlx::query(query).bind(post_id);
            let builder = if !user_username.is_empty() {
                builder.bind(user_username.clone())
            } else {
                builder
            };
            let result = builder.fetch_one(pool).await;

            debug!("query {query_str}\nresults {result:?}");

            let pg_row = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbPostGetByKeyErr::PostNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostGetByKeyErr::Db(err));
                }
            };

            let post = DbPostGet::from_row(&pg_row);
            let post = match post {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostGetByKeyErr::Db(err));
                }
            };

            let images_hashes = <(Vec<i64>,)>::from_row(&pg_row);
            let images_hashes = match images_hashes {
                Ok((images_hashes,)) => images_hashes,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostGetByKeyErr::Db(err));
                }
            };

            if post.state == POST_STATE_HIDDEN && post.user_username != user_username {
                return Err(DbPostGetByKeyErr::Unauthorized);
            }

            (images_hashes, post)
        };

        // get images
        let post = {
            let query = "SELECT
                                    image_hash,
                                    image_extension,
                                    image_kind,
                                    image_size_bytes,
                                    image_processed,
                                    image_used_count,
                                    image_width,
                                    image_height,
                                    image_modified_at,
                                    image_created_at
                                 FROM unnest($2)
                                  INNER JOIN files_images ON unnest=image_hash
                                 ";
            // (SELECT unnest(post_images_hashes) as post_image_hash
            //     FROM posts
            //     WHERE post_id = $1)

            let result = sqlx::query_as(query)
                .bind(post_id)
                .bind(&images_hashes)
                .fetch_all(pool)
                .await;

            let images: Vec<DbFileImage> = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostGetByKeyErr::Db(err));
                }
            };

            let images_sorted = images_hashes
                .clone()
                .into_iter()
                .map(|hash| images.iter().find(|img| img.hash == hash).cloned())
                .collect::<Option<Vec<DbFileImage>>>()
                .ok_or(DbPostGetByKeyErr::PostNotFound)?;

            let mut post: DbPostGet = post.into();

            post.images = images_sorted;

            post
        };

        Ok(post)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_get_by_id() {
    use catsquad_shared::PostState;

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

    db.post_update_image_add(0, user1.username.clone(), post1.id, 10, 555, "jpg", 10, 15)
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
        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id.clone())
            .await
            .unwrap();
        assert_eq!(post1.title, "title1");
        assert_eq!(post1.images[0].hash, 555);

        let result = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized)));
    }

    // pfp
    {
        db.post_update_state(
            0,
            user1.username.clone(),
            post1.id.clone(),
            PostState::Active,
        )
        .await
        .unwrap();

        assert_eq!(post1.user_pfp_hash, 0);
        assert_eq!(post1.user_pfp_proccesed, false);

        db.user_update_pfp(
            0,
            user1.username.clone(),
            async move |_, _| (),
            10,
            666,
            "webp",
            10,
            10,
        )
        .await
        .unwrap();

        let post1 = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.user_pfp_hash, 666);
        assert_eq!(post1.user_pfp_proccesed, false);

        db.file_image_update_proccsed(0, 666).await.unwrap();

        let post1 = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.user_pfp_hash, 666);
        assert_eq!(post1.user_pfp_proccesed, true);
    }

    // like
    {
        let post1 = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.liked, false);

        db.post_like_add(0, user2.username.clone(), post1.id)
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.liked, true);

        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.liked, false);

        let post1 = db
            .post_get_by_id("".to_string(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.liked, false);
    }
}
