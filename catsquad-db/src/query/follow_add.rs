use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbFollow {
    #[sqlx(rename = "follow_id")]
    pub id: i64,
    pub follower_user_username: String,
    pub following_user_username: String,
    #[sqlx(rename = "follow_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "follow_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbFollowAddErr {
    #[error("cant follow yourself")]
    CantFollowYourself,

    #[error("user is already followed")]
    UserAlreadyFollowed,

    #[error("user \"{0}\" was not found")]
    UserNotFound(String),

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn follow_define(&self) {
        let pool = &self.db;
        let query = "
            CREATE TABLE follow (
                follow_id int8 PRIMARY KEY generated always as identity,
                follower_user_username varchar NOT NULL references users(user_username) ON UPDATE CASCADE ON DELETE CASCADE,
                following_user_username varchar NOT NULL references users(user_username) ON UPDATE CASCADE ON DELETE CASCADE,
                follow_modified_at timestamp NOT NULL,
                follow_created_at timestamp NOT NULL
            );
            CREATE UNIQUE INDEX follow_follower_and_following_idx ON follow (following_user_username, follower_user_username);
        ";
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
    }

    pub async fn follow_add(
        &self,
        time: u64,
        follower_user_username: impl Into<String>,
        following_user_username: impl Into<String>,
    ) -> Result<DbFollow, DbFollowAddErr> {
        let pool = &self.db;
        let follower_user_username = follower_user_username.into();
        let following_user_username = following_user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        if following_user_username == follower_user_username {
            return Err(DbFollowAddErr::CantFollowYourself);
        }

        // add follow
        let follow_id = {
            let query = "
            INSERT INTO follow (
                    follower_user_username,
                    following_user_username,
                    follow_modified_at,
                    follow_created_at
                )
                VALUES ( $1, $2, $3, $3 )
                RETURNING follow_id
        ";

            let result = sqlx::query_as(query)
                .bind(&follower_user_username)
                .bind(&following_user_username)
                .bind(XTimestamp(time as i64))
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::Database(err))
                    if err.is_foreign_key_violation()
                        && err.constraint() == Some("follow_following_user_username_fkey") =>
                {
                    return Err(DbFollowAddErr::UserNotFound(following_user_username));
                }
                Err(sqlx::Error::Database(err))
                    if err.is_foreign_key_violation()
                        && err.constraint() == Some("follow_follower_user_username_fkey") =>
                {
                    return Err(DbFollowAddErr::UserNotFound(follower_user_username));
                }
                Err(sqlx::Error::Database(err))
                    if err.is_unique_violation()
                        && err.constraint() == Some("follow_follower_and_following_idx") =>
                {
                    return Err(DbFollowAddErr::UserAlreadyFollowed);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFollowAddErr::Db(err));
                }
            };

            let (follow_id,): (i64,) = result;

            follow_id
        };

        // update user following count
        {
            let query = "UPDATE users SET
                            user_following_count = user_following_count + 1,
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
                    return Err(DbFollowAddErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        // update user followers count
        {
            let query = "UPDATE users SET
                            user_followers_count = user_followers_count + 1,
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
                    return Err(DbFollowAddErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        let follow = DbFollow {
            id: follow_id,
            follower_user_username,
            following_user_username,
            modified_at: time,
            created_at: time,
        };

        Ok(follow)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_follow_add() {
    init_log();
    let db = Db::test_db(0, "test_follow_add").await;

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

    let result = db.follow_add(0, "hey", "hey2").await.unwrap();
    let user1 = db.user_get_by_username("", "hey").await.unwrap();
    let user2 = db.user_get_by_username("", "hey2").await.unwrap();
    let user3 = db.user_get_by_username("", "hey3").await.unwrap();
    assert_eq!(result.following_user_username, "hey2");
    assert_eq!(result.follower_user_username, "hey");
    assert_eq!(user1.following_count, 1);
    assert_eq!(user1.followers_count, 0);
    assert_eq!(user2.following_count, 0);
    assert_eq!(user2.followers_count, 1);
    assert_eq!(user3.following_count, 0);
    assert_eq!(user3.followers_count, 0);

    let result = db.follow_add(0, "hey3", "hey2").await.unwrap();
    let user1 = db.user_get_by_username("", "hey").await.unwrap();
    let user2 = db.user_get_by_username("", "hey2").await.unwrap();
    let user3 = db.user_get_by_username("", "hey3").await.unwrap();
    assert_eq!(result.following_user_username, "hey2");
    assert_eq!(result.follower_user_username, "hey3");
    assert_eq!(user1.following_count, 1);
    assert_eq!(user1.followers_count, 0);
    assert_eq!(user2.following_count, 0);
    assert_eq!(user2.followers_count, 2);
    assert_eq!(user3.following_count, 1);
    assert_eq!(user3.followers_count, 0);

    let result = db.follow_add(0, "hey2", "hey3").await.unwrap();
    let user1 = db.user_get_by_username("", "hey").await.unwrap();
    let user2 = db.user_get_by_username("", "hey2").await.unwrap();
    let user3 = db.user_get_by_username("", "hey3").await.unwrap();
    assert_eq!(result.following_user_username, "hey3");
    assert_eq!(result.follower_user_username, "hey2");
    assert_eq!(user1.following_count, 1);
    assert_eq!(user1.followers_count, 0);
    assert_eq!(user2.following_count, 1);
    assert_eq!(user2.followers_count, 2);
    assert_eq!(user3.following_count, 1);
    assert_eq!(user3.followers_count, 1);

    let result = db.follow_add(0, "hey", "hey3").await.unwrap();
    let user1 = db.user_get_by_username("", "hey").await.unwrap();
    let user2 = db.user_get_by_username("", "hey2").await.unwrap();
    let user3 = db.user_get_by_username("", "hey3").await.unwrap();
    assert_eq!(result.following_user_username, "hey3");
    assert_eq!(result.follower_user_username, "hey");
    assert_eq!(user1.following_count, 2);
    assert_eq!(user1.followers_count, 0);
    assert_eq!(user2.following_count, 1);
    assert_eq!(user2.followers_count, 2);
    assert_eq!(user3.following_count, 1);
    assert_eq!(user3.followers_count, 2);

    let result = db.follow_add(0, "hey", "hey2").await;
    assert!(matches!(result, Err(DbFollowAddErr::UserAlreadyFollowed)));

    let result = db.follow_add(0, "hey", "hey").await;
    assert!(matches!(result, Err(DbFollowAddErr::CantFollowYourself)));

    let result = db.follow_add(0, "hey", "he").await;
    assert!(matches!(result, Err(DbFollowAddErr::UserNotFound(_))));
    let result = db.follow_add(0, "hey0", "hey").await;
    assert!(matches!(result, Err(DbFollowAddErr::UserNotFound(_))));
    let result = db.follow_add(0, "h", "he").await;
    assert!(matches!(result, Err(DbFollowAddErr::UserNotFound(_))));
}
