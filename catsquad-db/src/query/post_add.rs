use crate::XTimestamp;
use catsquad_log::prelude::*;
use catsquad_shared::{
    POST_STATE_ACTIVE, POST_STATE_DRAFT, POST_STATE_HIDDEN, PostState, proccess_tags,
};
use sqlx::{AssertSqlSafe, SqlSafeStr};

use crate::{Db, DbUser};

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbPost {
    #[sqlx(rename = "post_id")]
    pub id: i64,
    #[sqlx(rename = "post_user_username")]
    pub user_username: String,
    #[sqlx(rename = "post_state")]
    pub state: String, // TODO optimize to be enum
    #[sqlx(rename = "post_title")]
    pub title: String,
    #[sqlx(rename = "post_description")]
    pub description: String,
    #[sqlx(rename = "post_tags")]
    pub tags: String,
    #[sqlx(rename = "post_likes_count")]
    #[sqlx(try_from = "i64")]
    pub likes_count: u32,
    #[sqlx(rename = "post_size_bytes")]
    #[sqlx(try_from = "i64")]
    pub size_bytes: u32,
    #[sqlx(rename = "post_images_hashes")]
    pub images_hashes: Vec<i64>,
    #[sqlx(rename = "post_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "post_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbPostAddErr {
    #[error("user not found")]
    UserNotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_define(&self) {
        let pool = &self.db;
        let query = "
            CREATE TABLE posts (
                post_id int8 PRIMARY KEY generated always as identity,
                post_user_username varchar(32) NOT NULL references users(user_username) ON UPDATE CASCADE ON DELETE CASCADE,
                post_state varchar(30) DEFAULT 'draft',
                post_title varchar(120),
                post_tags varchar(2000),
                post_description varchar(2000),
                post_likes_count int8 DEFAULT 0,
                post_size_bytes int8 DEFAULT 0,
                post_images_hashes int8[] DEFAULT array[]::int8[],
                post_modified_at timestamp NOT NULL,
                post_created_at timestamp NOT NULL
            );
            CREATE INDEX post_user_username_idx ON posts (post_user_username);
            CREATE INDEX post_username_and_state_idx ON posts (post_user_username, post_state);
        ";
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
    }

    pub async fn post_add(
        &self,
        time: u64,
        user_username: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        tags: impl Into<String>,
    ) -> Result<DbPost, DbPostAddErr> {
        // let user_id = create_user_id(user_key);
        let pool = &self.db;

        let input_user_username = user_username.into();
        let input_title = title.into();
        let input_description = description.into();
        let input_tags = proccess_tags(tags);

        let mut tx = pool.begin().await?;

        // get post draft if one exists
        let post: Option<DbPost> = {
            let query =
                "SELECT * FROM posts WHERE post_user_username=$1 AND post_state='draft' LIMIT 1";

            debug!("about to run {query}");

            let result = sqlx::query_as(query)
                .bind(&input_user_username)
                .fetch_one(&mut *tx)
                .await;

            match result {
                Ok(post) => Some(post),
                Err(sqlx::Error::RowNotFound) => None,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostAddErr::Db(err));
                }
            }
        };

        let Some(mut post) = post else {
            // add post
            let query = "
            INSERT INTO posts (
                    post_user_username,
                    post_title,
                    post_tags,
                    post_description,
                    post_modified_at,
                    post_created_at
                )
                VALUES ( $1, $2, $3, $4, $5, $5 )
                RETURNING post_id
        ";
            debug!("about to run {query}");
            let result = sqlx::query_as(query)
                .bind(&input_user_username)
                .bind(&input_title)
                .bind(&input_tags)
                .bind(&input_description)
                .bind(XTimestamp(time as i64))
                .fetch_one(&mut *tx)
                .await;

            let post = match result {
                Ok(post) => post,
                Err(sqlx::Error::Database(err))
                    if err.is_foreign_key_violation()
                        && err.constraint() == Some("posts_post_user_username_fkey") =>
                {
                    return Err(DbPostAddErr::UserNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostAddErr::Db(err));
                }
            };
            debug!("query: {query}\nresult: {post:#?}");

            let (post_id,): (i64,) = post;

            let post = DbPost {
                id: post_id,
                user_username: input_user_username,
                state: POST_STATE_DRAFT.to_string(),
                title: input_title,
                tags: input_tags,
                description: input_description,
                likes_count: 0,
                size_bytes: 0,
                images_hashes: Vec::new(),
                modified_at: time,
                created_at: time,
            };

            tx.commit().await?;

            return Ok(post);
        };

        // update post
        {
            let mut q_update = String::new();
            let input_offset = 3;
            let mut input_data = [""; 3];
            let mut input_index = 0;

            if !input_title.is_empty() && input_title != post.title {
                post.title = input_title.clone();

                q_update += "post_title = $";
                q_update += &(input_index + input_offset).to_string(); // TODO optimize somehow
                q_update += ",\n";
                input_data[input_index] = &input_title;
                input_index += 1;
            }

            if !input_description.is_empty() && input_description != post.description {
                post.description = input_description.clone();

                q_update += "post_description = $";
                q_update += &(input_index + input_offset).to_string(); // TODO optimize somehow
                q_update += ",\n";
                input_data[input_index] = &input_description;
                input_index += 1;
            }

            if !input_tags.is_empty() && input_tags != post.tags {
                post.tags = input_tags.clone();

                q_update += "post_tags = $";
                q_update += &(input_index + input_offset).to_string(); // TODO optimize somehow
                q_update += ",\n";
                input_data[input_index] = &input_tags;
                input_index += 1;
            }

            if input_index == 0 {
                return Ok(post);
            }

            let query_str = format!(
                "UPDATE posts SET
                    {q_update}
                    post_modified_at = $1
                    WHERE post_id = $2"
            );
            debug!("about to run {query_str}");

            let query = AssertSqlSafe(query_str.clone());
            let mut query_builder = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(post.id);

            trace!("query input {:#?}", input_data);

            for input in input_data {
                query_builder = query_builder.bind(input);
            }
            let result = query_builder.execute(&mut *tx).await;

            let post = match result {
                Ok(post) => post,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostAddErr::Db(err));
                }
            };

            debug!("query: {query_str}\nresult: {post:#?}");
        }

        tx.commit().await?;

        Ok(post)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_add() {
    init_log();
    let db = Db::test_db(0, "test_post_add").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token.clone(), 10, 10)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user.username.clone(), "title", "description", "tags")
        .await
        .unwrap();

    assert_eq!(post1.id, 1);
    assert_eq!(post1.user_username, "hey");
    assert_eq!(post1.state, PostState::Draft.to_string());
    assert_eq!(post1.title, "title");
    assert_eq!(post1.description, "description");
    assert_eq!(post1.tags, " tags ");
    assert_eq!(post1.likes_count, 0);
    assert_eq!(post1.size_bytes, 0);
    assert_eq!(post1.images_hashes, vec![] as Vec<i64>);
    assert_eq!(post1.modified_at, 0);
    assert_eq!(post1.created_at, 0);

    let post2 = db
        .post_add(0, user.username.clone(), "title2", "description2", "tags2")
        .await
        .unwrap();

    assert_eq!(post2.id, post1.id);
    assert_eq!(post2.state, PostState::Draft.to_string());
    assert_eq!(post2.title, "title2");
    assert_eq!(post2.description, "description2");
    assert_eq!(post2.tags, " tags2 ");

    let post2 = db
        .post_add(0, user.username.clone(), "title3", "", "")
        .await
        .unwrap();

    assert_eq!(post2.id, post1.id);
    assert_eq!(post2.state, PostState::Draft.to_string());
    assert_eq!(post2.title, "title3");
    assert_eq!(post2.description, "description2");
    assert_eq!(post2.tags, " tags2 ");

    let result = db
        .post_add(0, "invalid", "title", "description", "tags")
        .await;
    assert!(matches!(result, Err(DbPostAddErr::UserNotFound)));

    // tags are proccessed

    let post1 = db
        .post_add(
            0,
            user.username.clone(),
            "title",
            "description",
            " tAgs     tWo",
        )
        .await
        .unwrap();

    assert_eq!(post1.state, PostState::Draft.to_string());
    assert_eq!(post1.title, "title");
    assert_eq!(post1.description, "description");
    assert_eq!(post1.tags, " tags two ");
}
