use crate::{Db, XTimestamp, query::file_image_remove_tx::DbFileImageRemoveTxErr};
use catsquad_log::prelude::*;
use catsquad_shared::FileImage;

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdatePfpErr {
    #[error("image is same")]
    IsSame,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("image remove error {0}")]
    ImageRemove(#[from] DbFileImageRemoveTxErr),
}

impl Db {
    pub async fn user_update_pfp(
        &self,
        time: u64,
        user_username: impl Into<String>,
        callback_remove_image: impl AsyncFnOnce(i64, &str),
        file_size: u32,
        file_hash: i64,
        file_extension: impl Into<String>,
        file_width: u32,
        file_height: u32,
    ) -> Result<(), DbUserUpdatePfpErr> {
        let pool = &self.db;
        let user_username = user_username.into();
        let file_extension = file_extension.into();

        let mut tx = self
            .db
            .begin()
            .await
            .inspect_err(|err| error!("user_update_pfp {err}"))?;

        // get user
        let (current_pfp_hash, current_pfp_img_extension) = {
            let query = "SELECT user_pfp_image_hash, image_extension FROM users JOIN files_images ON user_pfp_image_hash=image_hash WHERE user_username = $1";

            let result = sqlx::query_as(query)
                .bind(&user_username)
                .fetch_one(&mut *tx)
                .await;

            let (current_pfp_hash, current_pfp_img_extension): (i64, String) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => (0, String::new()),
                Err(err) => {
                    error!("unexpected db error (post_update_file_add) {err}");
                    return Err(DbUserUpdatePfpErr::Db(err));
                }
            };
            (current_pfp_hash, current_pfp_img_extension)
        };

        if file_hash == current_pfp_hash {
            return Err(DbUserUpdatePfpErr::IsSame);
        }

        if current_pfp_hash != 0 {
            let (img_used_count, _) = self
                .file_image_remove_tx(&mut *tx, time, current_pfp_hash)
                .await?;
            if img_used_count == 0 {
                callback_remove_image(current_pfp_hash, current_pfp_img_extension.as_str()).await;
            }
        }

        let _ = self
            .file_image_add_tx(
                &mut *tx,
                time,
                file_hash,
                file_size,
                file_width,
                file_height,
                file_extension.as_str(),
            )
            .await?;

        let query = "UPDATE users SET
                            user_pfp_image_hash = $3,
                            user_modified_at = $1
                           WHERE user_username = $2";

        let result = sqlx::query(query)
            .bind(XTimestamp(time as i64))
            .bind(user_username)
            .bind(file_hash)
            .execute(pool)
            .await;

        debug!("query {query}\nresults {result:?}");

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserUpdatePfpErr::Db(err));
            }
        };

        tx.commit()
            .await
            .inspect_err(|err| error!("user_update_pfp {err}"))?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_update_pfp() {
    use crate::DbFileImageGetByHashErr;

    init_log();

    let db = Db::test_db(0, "test_user_update_pfp").await;

    let user1 = {
        let invite = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        db.user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
            .await
            .unwrap()
    };

    let user2 = {
        let invite = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        db.user_add(0, "hey2", "hey2", invite.token.clone(), 10, 10)
            .await
            .unwrap()
    };

    let result = db.file_image_get_by_hash(666).await;

    assert!(matches!(result, Err(DbFileImageGetByHashErr::NotFound)));
    assert_eq!(user1.pfp_image_hash, 0);

    let callback_remove_img = async move |file_hash: i64, file_extensoin: &str| {
        //
    };
    // try setting same pfp twice
    {
        for _ in 0..2 {
            let _result = db
                .user_update_pfp(
                    0,
                    user1.username.clone(),
                    callback_remove_img,
                    10,
                    666,
                    "pfp",
                    200,
                    300,
                )
                .await;

            let image = db.file_image_get_by_hash(666).await.unwrap();
            assert_eq!(image.hash, 666);
            assert_eq!(image.used_count, 1);

            let user1 = db
                .user_get_by_username(user1.username.clone())
                .await
                .unwrap();

            assert_eq!(user1.pfp_image_hash, 666);
        }
    }

    // try setting different pfp
    {
        let _result = db
            .user_update_pfp(
                0,
                user1.username.clone(),
                callback_remove_img,
                10,
                667,
                "pfp",
                200,
                300,
            )
            .await;

        let result = db.file_image_get_by_hash(666).await;
        assert!(matches!(result, Err(DbFileImageGetByHashErr::NotFound)));

        let image = db.file_image_get_by_hash(667).await.unwrap();
        assert_eq!(image.hash, 667);
        assert_eq!(image.used_count, 1);
    }

    // assert how it interacts with post images
    {
        let post1 = db
            .post_add(0, user1.username.clone(), "t1", "", "")
            .await
            .unwrap();

        let _img1 = db
            .post_update_image_add(0, user1.username.clone(), post1.id, 10, 667, "webp", 10, 10)
            .await
            .unwrap();

        let image = db.file_image_get_by_hash(667).await.unwrap();
        assert_eq!(image.hash, 667);
        assert_eq!(image.used_count, 2);

        let _result = db
            .user_update_pfp(
                0,
                user1.username.clone(),
                callback_remove_img,
                10,
                666,
                "pfp",
                200,
                300,
            )
            .await;

        let image = db.file_image_get_by_hash(667).await.unwrap();
        assert_eq!(image.hash, 667);
        assert_eq!(image.used_count, 1);

        let image = db.file_image_get_by_hash(666).await.unwrap();
        assert_eq!(image.hash, 666);
        assert_eq!(image.used_count, 1);
    }

    // assert user 2 adding same pfp
    {
        let _result = db
            .user_update_pfp(
                0,
                user2.username.clone(),
                callback_remove_img,
                10,
                666,
                "pfp",
                200,
                300,
            )
            .await;

        let image = db.file_image_get_by_hash(667).await.unwrap();
        assert_eq!(image.hash, 667);
        assert_eq!(image.used_count, 1);

        let image = db.file_image_get_by_hash(666).await.unwrap();
        assert_eq!(image.hash, 666);
        assert_eq!(image.used_count, 2);
    }
}
