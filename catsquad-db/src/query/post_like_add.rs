use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::PostState;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbPostLike {
    #[sqlx(rename = "post_like_id")]
    pub id: i64,
    #[sqlx(rename = "post_like_user_username")]
    pub user_username: String,
    #[sqlx(rename = "post_like_post_id")]
    pub post_id: i64,
    #[sqlx(rename = "post_like_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "post_like_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbPostLikeAddErr {
    #[error("cant like your own post")]
    CantLikeYourself,

    #[error("post was already liked")]
    PostWasAlreadyLiked,

    #[error("post \"{0}\" was not found")]
    PostNotFound(i64),

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_like_define(&self) {
        let pool = &self.db;
        // add foreign key FILE
        let query = "
            CREATE TABLE posts_likes (
                post_like_id int8 PRIMARY KEY generated always as identity,
                post_like_user_username varchar(32) NOT NULL references users(user_username) ON UPDATE CASCADE ON DELETE CASCADE,
                post_like_post_id int8 NOT NULL references posts(post_id) ON DELETE CASCADE,
                post_like_modified_at timestamp NOT NULL,
                post_like_created_at timestamp NOT NULL
            );
            CREATE INDEX post_like_username_idx ON posts_likes (post_like_user_username);
            CREATE INDEX post_like_username_and_post_id_idx ON posts_likes (post_like_user_username, post_like_post_id);
        ";
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
        // let query = "
        //         DEFINE TABLE post_like SCHEMAFULL;
        //         DEFINE FIELD user ON TABLE post_like TYPE record<user>;
        //         DEFINE FIELD post ON TABLE post_like TYPE record<post>;
        //         DEFINE FIELD modified_at ON TABLE post_like TYPE number;
        //         DEFINE FIELD created_at ON TABLE post_like TYPE number;
        //         DEFINE INDEX idx_user_post ON TABLE post_like COLUMNS user, post UNIQUE;
        //     ";
        // trace!("about to run {query}");
        // self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn post_like_add(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
    ) -> Result<DbPostLike, DbPostLikeAddErr> {
        let pool = &self.db;
        let user_username = user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get post like
        {
            let query = "SELECT EXISTS(SELECT 1 FROM posts_likes WHERE post_like_user_username=$1 AND post_like_post_id=$2)";

            let result = sqlx::query_as(query)
                .bind(&user_username)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostLikeAddErr::Db(err));
                }
            };

            let (exists,): (bool,) = result;

            if exists {
                return Err(DbPostLikeAddErr::PostWasAlreadyLiked);
            }
        }

        // get post
        {
            let query = "SELECT post_user_username, post_state FROM posts WHERE post_id = $1";

            let result = sqlx::query_as(query)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbPostLikeAddErr::PostNotFound(post_id));
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostLikeAddErr::Db(err));
                }
            };

            let (post_user_username, post_state): (String, String) = result;

            if user_username == post_user_username {
                return Err(DbPostLikeAddErr::CantLikeYourself);
            }

            let post_state = PostState::from(post_state);
            match post_state {
                PostState::Draft => return Err(DbPostLikeAddErr::Unauthorized),
                PostState::Hidden => return Err(DbPostLikeAddErr::Unauthorized),
                PostState::Active => (),
            }
        };

        // add post like
        let post_like_id = {
            let query = "
            INSERT INTO posts_likes (
                    post_like_user_username,
                    post_like_post_id,
                    post_like_modified_at,
                    post_like_created_at
                )
                VALUES ( $1, $2, $3, $3 )
                RETURNING post_like_id
        ";

            let result = sqlx::query_as(query)
                .bind(&user_username)
                .bind(post_id)
                .bind(XTimestamp(time as i64))
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::Database(err))
                    if err.is_foreign_key_violation()
                        && err.constraint() == Some("posts_likes_post_like_user_username_fkey") =>
                {
                    return Err(DbPostLikeAddErr::Unauthorized);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostLikeAddErr::Db(err));
                }
            };

            let (post_like_id,): (i64,) = result;

            post_like_id
        };

        // update post total like count
        {
            let query = "UPDATE posts SET
                            post_likes_count = post_likes_count + 1,
                            post_modified_at = $1
                            WHERE post_id = $2";

            debug!("about to run {query}");

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(post_id)
                .execute(&mut *tx)
                .await;

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostLikeAddErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        let post_like = DbPostLike {
            id: post_like_id,
            user_username,
            post_id,
            modified_at: time,
            created_at: time,
        };

        Ok(post_like)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_like_add() {
    use catsquad_shared::PostState;

    init_log();
    let db = Db::test_db(0, "test_post_like_add").await;

    let (user1, user2, user3) = {
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

    let (post1, post2) = {
        let post1 = db
            .post_add(0, user1.username.clone(), "title", "description", "tags")
            .await
            .unwrap();
        db.post_update_state(0, user1.username.clone(), post1.id, PostState::Active)
            .await
            .unwrap();

        let post2 = db
            .post_add(0, user1.username.clone(), "title2", "description", "tags")
            .await
            .unwrap();
        db.post_update_state(0, user1.username.clone(), post2.id, PostState::Hidden)
            .await
            .unwrap();

        assert_eq!(post1.likes_count, 0);
        assert_eq!(post2.likes_count, 0);

        (post1, post2)
    };

    // assert errors
    {
        let result = db.post_like_add(0, user1.username.clone(), post1.id).await;
        assert!(matches!(result, Err(DbPostLikeAddErr::CantLikeYourself)));

        let result = db.post_like_add(0, user2.username.clone(), post2.id).await;
        assert!(matches!(result, Err(DbPostLikeAddErr::Unauthorized)));

        let result = db.post_like_add(0, user1.username.clone(), 0).await;
        assert!(matches!(result, Err(DbPostLikeAddErr::PostNotFound(_))));
    }

    // assert success
    {
        db.post_update_state(0, user1.username.clone(), post2.id, PostState::Active)
            .await
            .unwrap();

        let post_like = db
            .post_like_add(0, user2.username.clone(), post1.id)
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user1.username.clone(), post1.id)
            .await
            .unwrap();

        assert_eq!(post1.likes_count, 1);
        assert_eq!(post_like.id, 1);
        assert_eq!(post_like.user_username, user2.username);
        assert_eq!(post_like.post_id, post1.id);
        assert_eq!(post_like.modified_at, 0);
        assert_eq!(post_like.created_at, 0);

        let post_like = db
            .post_like_add(1, user2.username.clone(), post2.id)
            .await
            .unwrap();

        let post2 = db
            .post_get_by_id(user2.username.clone(), post2.id)
            .await
            .unwrap();

        assert_eq!(post2.likes_count, 1);
        assert_eq!(post_like.id, 2);
        assert_eq!(post_like.user_username, user2.username);
        assert_eq!(post_like.post_id, post2.id);
        assert_eq!(post_like.modified_at, 1);
        assert_eq!(post_like.created_at, 1);

        let _post_like = db
            .post_like_add(1, user3.username.clone(), post2.id)
            .await
            .unwrap();

        let post2 = db
            .post_get_by_id(user3.username.clone(), post2.id)
            .await
            .unwrap();

        assert_eq!(post2.likes_count, 2);
    }

    // assert error
    {
        let result = db.post_like_add(0, user2.username.clone(), post1.id).await;
        assert!(matches!(result, Err(DbPostLikeAddErr::PostWasAlreadyLiked)));
    }
}
