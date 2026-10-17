use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::PostState;

#[derive(Debug, thiserror::Error)]
pub enum DbFollowRemoveErr {
    #[error("post not found")]
    NotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn follow_remove(
        &self,
        time: u64,
        follower_user_username: impl Into<String>,
        following_user_username: impl Into<String>,
    ) -> Result<(), DbFollowRemoveErr> {
        let pool = &self.db;

        let follower_user_username = follower_user_username.into();
        let following_user_username = following_user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("follow_remove {err}"))?;

        // delete post like
        {
            let query = "
                    DELETE FROM follow
                    WHERE
                        follower_user_username = $1
                        AND following_user_username = $2";

            let result = sqlx::query(query)
                .bind(&follower_user_username)
                .bind(&following_user_username)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFollowRemoveErr::Db(err));
                }
            };

            let affected = result.rows_affected();

            match affected {
                0 => return Err(DbFollowRemoveErr::NotFound),
                1 => (),
                _ => panic!("query is wrong, it must only delete 1 row max"),
            }
        }

        // update user following count
        {
            let query = "UPDATE users SET
                            user_following_count = user_following_count - 1,
                            user_modified_at = $1
                            WHERE user_username = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(&follower_user_username)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFollowRemoveErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        // update user followers count
        {
            let query = "UPDATE users SET
                            user_followers_count = user_followers_count - 1,
                            user_modified_at = $1
                            WHERE user_username = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(&following_user_username)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFollowRemoveErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_remove {err}"))?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_follow_remove() {
    init_log();
    let db = Db::test_db(0, "test_follow_remove").await;

    let _ = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user1 = db
            .user_add(0, "hey", "hey", invite1.token, 10, 10)
            .await
            .unwrap();
        let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        let user2 = db
            .user_add(0, "hey2", "hey", invite2.token, 10, 10)
            .await
            .unwrap();
        let invite3 = db.invite_add(0, "hey3@heyadora.com", 1).await.unwrap();
        let user3 = db
            .user_add(0, "hey3", "hey", invite3.token, 10, 10)
            .await
            .unwrap();

        (user1, user2, user3)
    };

    db.follow_add(0, "hey", "hey2").await.unwrap();
    db.follow_add(1, "hey", "hey3").await.unwrap();
    db.follow_add(2, "hey3", "hey2").await.unwrap();
    let user1 = db.user_get_by_username("", "hey").await.unwrap();
    let user2 = db.user_get_by_username("", "hey2").await.unwrap();
    let user3 = db.user_get_by_username("", "hey3").await.unwrap();
    assert_eq!(user1.following_count, 2);
    assert_eq!(user1.followers_count, 0);
    assert_eq!(user2.following_count, 0);
    assert_eq!(user2.followers_count, 2);
    assert_eq!(user3.following_count, 1);
    assert_eq!(user3.followers_count, 1);
    db.follow_remove(0, "hey", "hey2").await.unwrap();
    let user1 = db.user_get_by_username("", "hey").await.unwrap();
    let user2 = db.user_get_by_username("", "hey2").await.unwrap();
    let user3 = db.user_get_by_username("", "hey3").await.unwrap();
    assert_eq!(user1.following_count, 1);
    assert_eq!(user1.followers_count, 0);
    assert_eq!(user2.following_count, 0);
    assert_eq!(user2.followers_count, 1);
    assert_eq!(user3.following_count, 1);
    assert_eq!(user3.followers_count, 1);

    let followes = db.follow_get_all().await.unwrap();
    assert_eq!(followes.len(), 2);
    assert_eq!(followes[0].created_at, 2);
    assert_eq!(followes[1].created_at, 1);
}
