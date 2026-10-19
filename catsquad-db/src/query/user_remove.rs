use crate::{Db, DbFileImageRemoveTxErr};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserRemoveErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error(transparent)]
    Image(#[from] DbFileImageRemoveTxErr),
}

impl Db {
    pub async fn user_remove(
        &self,
        time: u64,
        mut callback_remove_image: impl AsyncFnMut(i64, &str),
        user_username: impl Into<String>,
    ) -> Result<(), DbUserRemoveErr> {
        let pool = &self.db;
        let user_username = user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("user_remove {err}"))?;

        // get posts images
        let posts_images = {
            let query = "SELECT post_images_hashes FROM posts
                               WHERE post_user_username = $1
                               ORDER BY post_created_at DESC";

            trace!("about to run {query}");

            let result = sqlx::query_as(query)
                .bind(&user_username)
                .fetch_all(&mut *tx)
                .await;

            let posts_images: Vec<(Vec<i64>,)> = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserRemoveErr::Db(err));
                }
            };

            posts_images
        };

        let mut delete_these_images = Vec::<(i64, String)>::new();

        // delete images
        {
            for (images,) in posts_images {
                for image_hash in images {
                    let (img_used_count, _img_size, extension) = self
                        .file_image_remove_tx(&mut *tx, time, image_hash)
                        .await?;

                    if img_used_count == 0 {
                        delete_these_images.push((image_hash, extension));
                    }
                }
            }
        }

        // delete user
        let (user_email, pfp_img_hash) = {
            let query = "DELETE FROM users
                                    WHERE user_username = $1
                                    RETURNING user_email, user_pfp_image_hash";

            let result = sqlx::query_as(query)
                .bind(user_username)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (user_email, pfp_img_hash): (String, i64) = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserRemoveErr::Db(err));
                }
            };
            (user_email, pfp_img_hash)
        };

        if pfp_img_hash != 0 {
            let (img_used_count, _img_size, extension) = self
                .file_image_remove_tx(&mut *tx, time, pfp_img_hash)
                .await?;

            if img_used_count == 0 {
                delete_these_images.push((pfp_img_hash, extension));
            }
        }

        // delete emails
        {
            let query = "DELETE FROM emails_sent
                                    WHERE email_sent_to_email = $1";

            let result = sqlx::query(query).bind(&user_email).execute(&mut *tx).await;

            debug!("query: {query}\nresult: {result:#?}");

            match result {
                Ok(_) => (),
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserRemoveErr::Db(err));
                }
            };
        }

        // delete emails
        {
            let query = "DELETE FROM invites
                                    WHERE invite_email = $1";

            let result = sqlx::query(query).bind(&user_email).execute(&mut *tx).await;

            debug!("query: {query}\nresult: {result:#?}");

            match result {
                Ok(_) => (),
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserRemoveErr::Db(err));
                }
            };
        }

        //

        tx.commit()
            .await
            .inspect_err(|err| error!("user_remove {err}"))?;

        for (hash, extension) in delete_these_images {
            callback_remove_image(hash, extension.as_str()).await;
        }

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_remove() {
    use catsquad_shared::PostState;

    init_log();
    let db = Db::test_db(0, "test_user_remove").await;

    let (user, user2, user3) = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 100).await.unwrap();
        let invite2 = db.invite_add(0, "hey2@heyadora.com", 100).await.unwrap();
        let invite3 = db.invite_add(0, "hey3@heyadora.com", 100).await.unwrap();
        let user = db
            .user_add(0, "hey", "hey", invite1.token, 100, 10)
            .await
            .unwrap();
        let user2 = db
            .user_add(1, "hey2", "hey", invite2.token, 100, 10)
            .await
            .unwrap();
        let user3 = db
            .user_add(2, "hey3", "hey", invite3.token, 100, 10)
            .await
            .unwrap();

        (user, user2, user3)
    };

    let callback_remove = async move |_hash: i64, _extension: &str| {
        //
    };

    // add bunch of data
    {
        // add sessions
        {
            db.session_add(0, "hey@heyadora.com").await.unwrap();
            db.session_add(1, "hey2@heyadora.com").await.unwrap();
            db.session_add(2, "hey3@heyadora.com").await.unwrap();
        }

        // add email change
        {
            db.email_change_add(0, user.username.clone(), 10)
                .await
                .unwrap();

            db.email_change_add(0, user2.username.clone(), 10)
                .await
                .unwrap();
        }

        // add emails sents
        {
            db.email_sent_add(
                0,
                crate::DbEmailSentReason::UserEmailChangeAddNew,
                "hey@heyadora.com",
                "hello",
            )
            .await
            .unwrap();

            db.email_sent_add(
                0,
                crate::DbEmailSentReason::UserEmailChangeAddNew,
                "hey2@heyadora.com",
                "hello2",
            )
            .await
            .unwrap();
        }

        // add post1
        {
            let post1 = db
                .post_add(0, user.username.clone(), "title", "description", "tags")
                .await
                .unwrap();

            db.post_update_state(0, user.username.clone(), post1.id, PostState::Active)
                .await
                .unwrap();

            db.post_update_image_add(1, user.username.clone(), post1.id, 1, 666, "png", 15, 15)
                .await
                .unwrap();

            db.post_update_image_add(2, user.username.clone(), post1.id, 1, 667, "png", 15, 15)
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
        }

        // add post2
        {
            let post2 = db
                .post_add(1, user2.username.clone(), "title2", "description", "tags")
                .await
                .unwrap();
            db.post_update_state(0, user2.username.clone(), post2.id, PostState::Active)
                .await
                .unwrap();

            db.post_like_add(0, user.username.clone(), post2.id)
                .await
                .unwrap();

            db.post_like_add(0, user3.username.clone(), post2.id)
                .await
                .unwrap();

            let comment1 = db
                .comment_add(0, user.username.clone(), post2.id, None, "one2")
                .await
                .unwrap();

            db.comment_add(
                0,
                user2.username.clone(),
                post2.id,
                Some(comment1.id),
                "one2",
            )
            .await
            .unwrap();

            db.post_update_image_add(3, user2.username.clone(), post2.id, 10, 666, "webp", 10, 10)
                .await
                .unwrap();

            db.post_update_image_add(4, user2.username.clone(), post2.id, 10, 668, "webp", 10, 10)
                .await
                .unwrap();
        }

        // add post3
        {
            let _post3 = db
                .post_add(2, user3.username.clone(), "title3", "description", "tags")
                .await
                .unwrap();
        }

        // add pfp
        for username in [user.username.clone(), user2.username.clone()] {
            db.user_update_pfp(5, username, callback_remove, 10, 500, "webp", 10, 10)
                .await
                .unwrap();
        }

        // add follow
        {
            db.follow_add(0, user.username.clone(), user2.username.clone())
                .await
                .unwrap();
            db.follow_add(0, user2.username.clone(), user.username.clone())
                .await
                .unwrap();
            db.follow_add(0, user3.username.clone(), user2.username.clone())
                .await
                .unwrap();
        }

        // add password change
        {
            db.password_change_add(0, user.email.clone(), 100)
                .await
                .unwrap();

            db.password_change_add(0, user2.email.clone(), 100)
                .await
                .unwrap();
        }
    }

    // assert success
    {
        let users = db.user_get_all().await.unwrap();
        let posts = db.post_get_all().await.unwrap();
        let comments = db.comment_get_all().await.unwrap();
        let likes = db.post_like_get_all().await.unwrap();
        let sessions = db.sessoin_get_all().await.unwrap();
        let email_change = db.email_change_get_all().await.unwrap();
        let emails_sents = db.email_sent_get_all().await.unwrap();
        let images = db.file_image_get_all().await.unwrap();
        let follows = db.follow_get_all().await.unwrap();
        let invites = db.invite_get_all().await.unwrap();
        let password_change = db.password_change_get_all().await.unwrap();

        assert_eq!(password_change.len(), 2);
        assert_eq!(invites.len(), 3);
        assert_eq!(follows.len(), 3);
        assert_eq!(images.len(), 4);
        assert_eq!(emails_sents.len(), 2);
        assert_eq!(email_change.len(), 2);
        assert_eq!(users.len(), 3);
        assert_eq!(posts.len(), 3);
        assert_eq!(comments.len(), 4);
        assert_eq!(likes.len(), 3);
        assert_eq!(sessions.len(), 3);

        assert_eq!(images[0].hash, 500);
        assert_eq!(images[0].used_count, 2);

        assert_eq!(images[1].hash, 668);
        assert_eq!(images[1].used_count, 1);

        assert_eq!(images[2].hash, 667);
        assert_eq!(images[2].used_count, 1);

        assert_eq!(images[3].hash, 666);
        assert_eq!(images[3].used_count, 2);

        let result = db.user_remove(0, callback_remove, "hey").await;
        assert!(matches!(result, Ok(_)));

        let users = db.user_get_all().await.unwrap();
        let posts = db.post_get_all().await.unwrap();
        let comments = db.comment_get_all().await.unwrap();
        let likes = db.post_like_get_all().await.unwrap();
        let sessions = db.sessoin_get_all().await.unwrap();
        let email_change = db.email_change_get_all().await.unwrap();
        let emails_sents = db.email_sent_get_all().await.unwrap();
        let images = db.file_image_get_all().await.unwrap();
        let follows = db.follow_get_all().await.unwrap();
        let invites = db.invite_get_all().await.unwrap();
        let password_change = db.password_change_get_all().await.unwrap();

        assert_eq!(password_change.len(), 1);
        assert_eq!(invites.len(), 2);
        assert_eq!(follows.len(), 1);
        assert_eq!(images.len(), 3);
        assert_eq!(emails_sents.len(), 1);
        assert_eq!(email_change.len(), 1);
        assert_eq!(users.len(), 2);
        assert_eq!(posts.len(), 2);
        assert_eq!(comments.len(), 0);
        assert_eq!(likes.len(), 1);
        assert_eq!(sessions.len(), 2);

        assert_eq!(users[0].username, "hey3");
        assert_eq!(users[1].username, "hey2");
        assert_eq!(posts[0].title, "title3");
        assert_eq!(posts[1].title, "title2");

        assert_eq!(images[0].hash, 500);
        assert_eq!(images[0].used_count, 1);

        assert_eq!(images[1].hash, 668);
        assert_eq!(images[1].used_count, 1);

        assert_eq!(images[2].hash, 666);
        assert_eq!(images[2].used_count, 1);
    }
}
