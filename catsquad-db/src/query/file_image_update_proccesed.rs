use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPostUpdateProccesedErr {
    #[error("not found")]
    NotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal error {0}")]
    InternalError(String),
}

impl Db {
    pub async fn post_update_proccsed(
        &self,
        time: u64,
        file_hash: i64,
    ) -> Result<(), DbPostUpdateProccesedErr> {
        let pool = &self.db;

        let query = "UPDATE files_images SET
                            image_processed = TRUE,
                            image_modified_at = $1
                            WHERE image_hash = $2";

        let result = sqlx::query(query)
            .bind(XTimestamp(time as i64))
            .bind(file_hash)
            .execute(pool)
            .await;

        debug!("query: {query}\nresult: {result:#?}");

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbPostUpdateProccesedErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostUpdateProccesedErr::Db(err));
            }
        };

        let affected = result.rows_affected();
        if affected != 1 {
            let error = format!(
                "something is wrong, updated wrong number of rows, expected 1, got {}, query {}, params {} {}",
                affected, query, file_hash, time
            );
            error!("{error}");
            return Err(DbPostUpdateProccesedErr::InternalError(error));
        }
        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_proccesed() {
    use crate::{DbPost, DbUser};

    init_log();

    let db = Db::test_db(0, "test_post_update_proccesed").await;

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

    let (post, post2) = {
        use catsquad_shared::PostState;

        let post = db
            .post_add(0, user.username.clone(), "title", "description", "")
            .await
            .unwrap();
        db.post_update_state(0, user.username.clone(), post.id, PostState::Active)
            .await
            .unwrap();

        let post2 = db
            .post_add(1, user2.username.clone(), "title2", "description", "")
            .await
            .unwrap();
        db.post_update_state(0, user2.username.clone(), post2.id, PostState::Active)
            .await
            .unwrap();

        (post, post2)
    };

    // add images
    {
        let add_post_file_fn =
            async |time: u64, user: &DbUser, post: &DbPost, hash: i64, size: u32| {
                db.post_update_file_add(
                    time,
                    user.username.clone(),
                    post.id,
                    size,
                    hash,
                    "png",
                    50,
                    50,
                )
                .await
            };
        add_post_file_fn(3, &user, &post, 10, 1).await.unwrap();
        add_post_file_fn(4, &user, &post, 20, 1).await.unwrap();
        add_post_file_fn(5, &user2, &post2, 10, 1).await.unwrap();
    }

    // assert success
    {
        let images = db.post_get_unproccesed().await.unwrap();
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].hash, 10);
        assert_eq!(images[0].processed, false);
        assert_eq!(images[0].used_count, 2);
        assert_eq!(images[1].hash, 20);
        assert_eq!(images[1].processed, false);
        assert_eq!(images[1].used_count, 1);

        db.post_update_proccsed(6, 10).await.unwrap();

        let images = db.post_get_unproccesed().await.unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].hash, 20);
        assert_eq!(images[0].processed, false);
        assert_eq!(images[0].used_count, 1);
    }
}
