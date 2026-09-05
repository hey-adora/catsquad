use crate::{Db, DbFileImage, DbInvite, DbPost};
use catsquad_log::prelude::*;
use catsquad_shared::{POST_STATE_DRAFT, POST_STATE_HIDDEN, PostState};
use sqlx::{FromRow, Row, any::AnyRow, postgres::PgRow};

#[derive(Debug, thiserror::Error)]
pub enum DbPostImageGetByHashErr {
    #[error("post not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_image_get_by_hash(
        &self,
        user_username: String,
        post_id: i64,
        image_hash: i64,
    ) -> Result<DbFileImage, DbPostImageGetByHashErr> {
        let pool = &self.db;

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
                                image_created_at,
                                post_user_username,
                                post_state
                            FROM (SELECT unnest(post_images_hashes) as post_image_hash, post_state, post_user_username FROM posts WHERE post_id = $1)
                            INNER JOIN files_images
                                ON post_image_hash=image_hash
                            WHERE image_hash = $2
                            LIMIT 1";
        // INNER JOIN posts ON comment_post_id = post_id
        // let query = "SELECT * FROM posts WHERE post_id = $1 AND post_state != 'draft'";

        // debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(post_id)
            .bind(image_hash)
            .fetch_one(pool)
            .await;

        debug!("query {query}\nresults {result:?}");

        let row = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbPostImageGetByHashErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostImageGetByHashErr::Db(err));
            }
        };

        let post_user_username: String = row.get("post_user_username");
        let post_state: String = row.get("post_state");

        // row

        let post_state = PostState::from(post_state);
        match post_state {
            PostState::Hidden if post_user_username == user_username => (),
            PostState::Active => (),
            _ => return Err(DbPostImageGetByHashErr::Unauthorized),
        }

        let image = DbFileImage::from_row(&row)
            .inspect_err(|err| error!("post_image_get_by_hash from_row {}", err.to_string()))?;
        // let image: DbFileImage = row.try_into();
        // if post.state == POST_STATE_HIDDEN && post.user_username != user_username {
        //     return Err(DbPostImageGetByHashErr::Unauthorized);
        // }

        Ok(image)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_image_get_by_hash() {
    // TODO finish the error testing stuff
    init_log();

    let db = Db::test_db(0, "test_post_image_get_by_hash").await;

    let (user1, user2) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user1 = db
            .user_add(0, "hey", "hey", invite1.token.clone(), 100, 100)
            .await
            .unwrap();

        let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite2.token.clone(), 100, 100)
            .await
            .unwrap();

        (user1, user2)
    };

    let post1 = db
        .post_add(0, user1.username.clone(), "title1", "description1", "tags")
        .await
        .unwrap();

    db.post_update_state(0, user1.username.clone(), post1.id, PostState::Active)
        .await
        .unwrap();

    let image = db
        .post_update_file_add(0, user1.username.clone(), post1.id, 19, 666, "ico", 10, 15)
        .await
        .unwrap();

    let image2 = db
        .post_update_file_add(0, user1.username.clone(), post1.id, 29, 266, "png", 20, 25)
        .await
        .unwrap();

    let result = db
        .post_image_get_by_hash(user1.username.clone(), post1.id, 666)
        .await
        .unwrap();
    assert_eq!(image.hash, result.hash);

    // {
    //     let result = db
    //         .post_get_by_id(user1.username.clone(), post1.id.clone())
    //         .await;
    //     assert!(matches!(result, Err(DbPostGetByKeyErr::PostNotFound)));

    //     let result = db
    //         .post_get_by_id(user2.username.clone(), post1.id.clone())
    //         .await;
    //     assert!(matches!(result, Err(DbPostGetByKeyErr::PostNotFound)));
    // }

    // db.post_update_state(
    //     0,
    //     user1.username.clone(),
    //     post1.id.clone(),
    //     PostState::Active,
    // )
    // .await
    // .unwrap();

    // {
    //     let result = db
    //         .post_get_by_id(user1.username.clone(), post1.id.clone())
    //         .await;
    //     assert!(matches!(result, Ok(_)));

    //     let result = db
    //         .post_get_by_id(user2.username.clone(), post1.id.clone())
    //         .await;
    //     assert!(matches!(result, Ok(_)));
    // }

    // db.post_update_state(
    //     0,
    //     user1.username.clone(),
    //     post1.id.clone(),
    //     PostState::Hidden,
    // )
    // .await
    // .unwrap();

    // {
    //     let result = db
    //         .post_get_by_id(user1.username.clone(), post1.id.clone())
    //         .await;
    //     assert!(matches!(result, Ok(_)));

    //     let result = db
    //         .post_get_by_id(user2.username.clone(), post1.id.clone())
    //         .await;
    //     assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized)));
    // }
}
