use crate::{Db, DbUser, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::PostState;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbComment {
    #[sqlx(rename = "comment_id")]
    pub id: i64,
    #[sqlx(rename = "comment_user_username")]
    pub user_username: String,
    #[sqlx(rename = "comment_post_id")]
    pub post_id: i64,
    #[sqlx(rename = "comment_replies_count")]
    pub replies_count: usize,
    #[sqlx(rename = "comment_parents")]
    pub parents: Vec<i64>,
    #[sqlx(rename = "comment_text")]
    pub text: String,
    #[sqlx(rename = "comment_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "comment_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbCommentAddErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("post \"{0}\" was not found")]
    PostNotFound(i64),

    #[error("user \"{0}\" was not found")]
    UserNotFound(String),

    #[error("reply_comment \"{0}\" was not found")]
    ParentNotFound(i64),

    #[error("unauthorized")]
    Unauthorized,
}

impl Db {
    pub async fn comment_define(&self) {
        let pool = &self.db;
        let query = "
            CREATE TABLE comments (
                comment_id int8 PRIMARY KEY generated always as identity,
                comment_user_username varchar NOT NULL references users(user_username) ON UPDATE CASCADE ON DELETE CASCADE,
                comment_post_id int8 NOT NULL references posts(post_id),
                comment_replies_count int8 DEFAULT 0,
                comment_parents int8[] DEFAULT array[]::int8[],
                comment_text text NOT NULL,
                comment_modified_at timestamp NOT NULL,
                comment_created_at timestamp NOT NULL
            );
            CREATE INDEX comment_user_username_idx ON comments (comment_user_username);
        ";
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
        // let query = "
        //         DEFINE TABLE comment SCHEMAFULL;
        //         DEFINE FIELD user ON TABLE comment TYPE record<user>;
        //         DEFINE FIELD post ON TABLE comment TYPE record<post>;
        //         DEFINE FIELD parent ON TABLE comment TYPE array<record<comment>>;
        //         DEFINE FIELD replies_count ON TABLE comment TYPE number;
        //         DEFINE FIELD text ON TABLE comment TYPE string;
        //         DEFINE FIELD modified_at ON TABLE comment TYPE number;
        //         DEFINE FIELD created_at ON TABLE comment TYPE number;
        //         DEFINE INDEX idx_comment_parent ON TABLE comment COLUMNS parent;
        //     ";
        // trace!("about to run {query}");
        // self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn comment_add(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
        comment_parent_id: Option<i64>,
        text: impl Into<String>,
    ) -> Result<DbComment, DbCommentAddErr> {
        let pool = &self.db;
        let user_username = user_username.into();
        let text = text.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get post
        {
            let query = "SELECT post_user_username, post_state FROM posts WHERE post_id = $1";

            let result = sqlx::query_as(query)
                .bind(post_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (post_user_username, post_state): (String, String) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbCommentAddErr::PostNotFound(post_id));
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbCommentAddErr::Db(err));
                }
            };

            let post_state = PostState::from(post_state);
            match post_state {
                PostState::Hidden if post_user_username == user_username => (),
                PostState::Active => (),
                _ => return Err(DbCommentAddErr::Unauthorized),
            }
        }

        // if exists: update parent and parent id's
        let parent_ids = if let Some(parent_id) = comment_parent_id {
            // let query = "SELECT comment_parents FROM comments WHERE comment_id=$1";
            let query = "UPDATE comments SET
                               comment_replies_count = comment_replies_count + 1,
                               comment_modified_at = $2
                               WHERE comment_id=$1
                               RETURNING comment_parents";
            // let query = "SELECT EXISTS(SELECT 1 FROM comments WHERE comment_id=$1)";

            //comment_parents
            // comment_replies_count

            let result = sqlx::query_as(query)
                .bind(parent_id)
                .bind(XTimestamp(time as i64))
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (parent_ids,): (Vec<i64>,) = match result {
                Ok(v) => v,
                // Ok((false,)) => return Err(DbCommentAddErr::ParentNotFound(parent_id)),
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbCommentAddErr::ParentNotFound(parent_id));
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbCommentAddErr::Db(err));
                }
            };

            parent_ids
        } else {
            Vec::new()
        };

        // insert comment
        let comment_id = {
            let query = "
                    INSERT INTO comments (
                            comment_user_username,
                            comment_post_id,
                            comment_parents,
                            comment_text,
                            comment_modified_at,
                            comment_created_at
                        )
                        VALUES ( $1, $2, $3, $4, $5, $5 )
                        RETURNING comment_id
                    ";

            let result = sqlx::query_as(query)
                .bind(&user_username)
                .bind(post_id)
                .bind(&parent_ids)
                .bind(&text)
                .bind(XTimestamp(time as i64))
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (comment_id,): (i64,) = match result {
                Ok(v) => v,
                Err(sqlx::Error::Database(err))
                    if err.is_foreign_key_violation()
                        && err.constraint() == Some("comments_comment_user_username_fkey") =>
                {
                    return Err(DbCommentAddErr::UserNotFound(user_username));
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbCommentAddErr::Db(err));
                }
            };

            comment_id
        };

        tx.commit()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        let comment = DbComment {
            id: comment_id,
            user_username,
            post_id,
            replies_count: 0,
            parents: parent_ids,
            text,
            modified_at: time,
            created_at: time,
        };

        Ok(comment)
        // TODO check if user exists in other queries too
        // let post_key = post_key.into();
        // let post_id = create_post_id(post_key.clone());
        // let parent_id = comment_parent_key.map(|v| create_comment_id(v));

        // IF $post.user != $user_id {
        //     THROW "unauthorized"
        // };

        // let query = r#"
        //          BEGIN TRANSACTION;

        //          LET $post = SELECT NONE FROM ONLY $post_id;
        //          LET $user = SELECT NONE FROM ONLY $user_id;

        //          IF !$user {
        //              THROW "user not found"
        //          };

        //          IF !$post {
        //              THROW "not found"
        //          };

        //          LET $parent = IF $parent_id {
        //                  SELECT id, parent, replies_count FROM ONLY $parent_id
        //              } ELSE {
        //                  NULL
        //              };

        //          IF $parent_id AND !$parent {
        //              THROW "parent not found"
        //          };

        //          IF $parent {
        //             UPDATE $parent.id SET replies_count = $parent.replies_count + 1;
        //          };

        //          LET $parent = if $parent {
        //                 if $parent.parent { $parent.parent } else { [] } + [$parent.id]
        //             } else {
        //                 []
        //             };

        //          CREATE comment SET
        //             user = $user_id,
        //             post = $post_id,
        //             parent = $parent,
        //             replies_count = 0,
        //             text = $comment_text,
        //             modified_at = $time,
        //             created_at = $time
        //          RETURN *, user.*;

        //          COMMIT TRANSACTION;
        //         "#;
        // trace!("about to run {query}");
        // self.db
        //     .query(query)
        //     .bind(("time", time))
        //     .bind(("user_id", user_id.clone()))
        //     .bind(("post_id", post_id.clone()))
        //     .bind(("comment_text", text.into()))
        //     .bind(("parent_id", parent_id.clone()))
        //     .await
        //     .check_better(|err| match err {
        //         err if err.thrown("not found") => DbCommentAddErr::PostNotFound(post_key.to_sql()),
        //         err if err.thrown("user not found") => {
        //             DbCommentAddErr::UserNotFound(user_id.key.to_sql())
        //         }
        //         err if err.thrown("parent not found") => DbCommentAddErr::ParentNotFound(
        //             parent_id
        //                 .map(|v| v.key.to_sql())
        //                 .unwrap_or_else(|| "invalid".to_string()),
        //         ),
        //         err => {
        //             error!("unexpected db error {err}");
        //             DbCommentAddErr::Db(err)
        //         }
        //     })
        //     .and_then_take_expect(9)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_comment_add() {
    // use crate::create_user_id;
    // TODO check un-authorized errors

    init_log();
    let db = Db::test_db(0, "test_comment_add").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user.username.clone(), "title", "description", "tags")
        .await
        .unwrap();
    db.post_update_state(0, user.username.clone(), post1.id, PostState::Active)
        .await
        .unwrap();

    let comment1 = db
        .comment_add(0, user.username.clone(), post1.id, None, "one")
        .await
        .unwrap();

    let result = db
        .comment_add(0, user.username.clone(), 0, None, "one1")
        .await;
    assert!(matches!(result, Err(DbCommentAddErr::PostNotFound(_))));

    let result = db.comment_add(0, "invalid", post1.id, None, "one2").await;
    assert!(matches!(result, Err(DbCommentAddErr::UserNotFound(_))));

    let result = db
        .comment_add(0, user.username.clone(), post1.id, Some(0), "one2")
        .await;
    assert!(matches!(result, Err(DbCommentAddErr::ParentNotFound(_))));

    let _comment2 = db
        .comment_add(
            1,
            user.username.clone(),
            post1.id,
            Some(comment1.id),
            "one3",
        )
        .await
        .unwrap();

    // let comments = db.comment_get_all().await.unwrap();

    // assert_eq!(comments.len(), 2);
    // assert_eq!(comments[0].parent.len(), 1);
    // assert_eq!(comments[0].parent[0], comment1.id.clone());
    // assert_eq!(comments[0].text, "one3");
    // assert_eq!(comments[1].parent.len(), 0);
    // assert_eq!(comments[1].text, "one");
}
