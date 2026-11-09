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
    #[sqlx(rename = "post_user_id")]
    pub user_id: i64,
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
    #[sqlx(rename = "post_user_followers")]
    #[sqlx(try_from = "i64")]
    pub followers: u32,
    #[sqlx(rename = "post_user_followed")]
    pub followed: bool,
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

impl Db {
    pub async fn post_get_by_id(
        &self,
        user_username: String,
        post_id: i64,
    ) -> Result<DbPostGet, DbPostGetByKeyErr> {
        let pool = &self.db;

        let (
            q_join_my_user,
            //
            q_liked_by_me_join,
            q_liked_by_me_field,
            //
            q_followed_by_me_join,
            q_followed_by_me_field,
        ) = match user_username.is_empty() {
            true => (
                "",
                //
                "",
                "false as post_liked",
                //
                "",
                "false as post_user_followed",
            ),
            false => (
                "JOIN users my_user ON $2=my_user.user_username",
                //
                "LEFT JOIN posts_likes ON post_id=post_like_post_id AND post_like_user_id = my_user.user_id",
                "coalesce(post_like_post_id > 0, false) as post_liked",
                //
                "LEFT JOIN follow ON my_user.user_id=follower_user_id AND post_user.user_id=following_user_id",
                "coalesce(follow_id > 0, false) as post_user_followed",
            ),
        };

        // get post
        let (images_hashes, mut post) = {
            let query_str = format!(
                "
                            SELECT
                                    post_images_hashes,

                                    post_user.user_pfp_image_hash as post_user_pfp_hash,
                                    coalesce(image_processed, false) as post_user_pfp_proccesed,

                                    {q_liked_by_me_field},

                                    post_user.user_followers_count as post_user_followers,
                                    {q_followed_by_me_field},

                                    post_id,
                                    post_user_id,
                                    post_user.user_username as post_user_username,
                                    post_state,
                                    post_title,
                                    post_description,
                                    post_tags,
                                    post_likes_count,
                                    post_size_bytes,
                                    post_modified_at,
                                    post_created_at
                                FROM posts

                                JOIN users post_user ON post_user_id=post_user.user_id
                                LEFT JOIN files_images ON user_pfp_image_hash=image_hash

                                {q_join_my_user}
                                {q_liked_by_me_join}
                                {q_followed_by_me_join}

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
        {
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
                .bind(post_id) // TODO i think this is useless
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

            post.images = images_sorted;
        }

        Ok(post)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_get_by_id() {
    use catsquad_shared::PostState;

    init_log();

    let db = Db::test_db(0, "test_post_get_by_id").await;

    // register user1 and user2
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

    // create post
    let post1 = {
        let post1 = db
            .post_add(0, user1.username.clone(), "title1", "description1", "tags")
            .await
            .unwrap();

        db.post_update_image_add(0, user1.username.clone(), post1.id, 10, 555, "jpg", 10, 15)
            .await
            .unwrap();

        post1
    };

    // asssert error
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

    // assert success when post is active
    {
        db.post_update_state(
            0,
            user1.username.clone(),
            post1.id.clone(),
            PostState::Active,
        )
        .await
        .unwrap();

        let result = db
            .post_get_by_id(user1.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));

        let result = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));
    }

    // asssert succuess/error when post is hidden
    {
        db.post_update_state(
            0,
            user1.username.clone(),
            post1.id.clone(),
            PostState::Hidden,
        )
        .await
        .unwrap();

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

    // assert pfp state of post author
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

    // assert post like state
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

    // assert follow state
    {
        let post1 = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.followed, false);

        db.follow_add(0, user2.username.clone(), user1.username.clone())
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user2.username.clone(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.followers, 1);
        assert_eq!(post1.followed, true);

        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.followed, false);

        let post1 = db
            .post_get_by_id("".to_string(), post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.followed, false);
    }
}
