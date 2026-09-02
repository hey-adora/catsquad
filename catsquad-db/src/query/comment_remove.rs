use crate::{Db, DbComment, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::PostState;

#[derive(Debug, thiserror::Error)]
pub enum DbCommentRemoveErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("comment \"{0}\" was not found")]
    CommentNotFound(i64),

    // #[error("user \"{0}\" was not found")]
    // UserNotFound(String),
    #[error("unauthorized")]
    Unauthorized,
}

impl Db {
    pub async fn comment_remove(
        &self,
        time: u64,
        user_username: impl Into<String>,
        comment_id: i64,
    ) -> Result<(), DbCommentRemoveErr> {
        let pool = &self.db;
        let user_username = user_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get post
        let comment_parents = {
            let query = "SELECT
                                comment_parents,
                                comment_user_username,
                                post_user_username,
                                post_state
                                FROM comments
                                INNER JOIN posts ON comment_post_id = post_id
                                WHERE comment_id = $1";

            let result = sqlx::query_as(query)
                .bind(comment_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let (comment_parents, commet_user_username, post_user_username, post_state): (
                Vec<i64>,
                String,
                String,
                String,
            ) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbCommentRemoveErr::CommentNotFound(comment_id));
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbCommentRemoveErr::Db(err));
                }
            };

            let post_state = PostState::from(post_state);

            if user_username != commet_user_username {
                return Err(DbCommentRemoveErr::Unauthorized);
            }

            let post_state = PostState::from(post_state);
            match post_state {
                PostState::Hidden if post_user_username == user_username => (),
                PostState::Active => (),
                _ => return Err(DbCommentRemoveErr::Unauthorized),
            }

            comment_parents
        };

        // update parent
        if let Some(parent_id) = comment_parents.last() {
            let query = "UPDATE comments SET
                            comment_replies_count = comment_replies_count - 1,
                            comment_modified_at = $1
                            WHERE comment_id = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(parent_id)
                .execute(&mut *tx)
                .await;

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbCommentRemoveErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        // remove comment
        {
            let query = "DELETE FROM comments WHERE comment_id = $1 OR $1 = ANY(comment_parents)";

            let result = sqlx::query(query).bind(comment_id).execute(&mut *tx).await;

            debug!("query: {query}\nresult: {result:#?}");

            match result {
                Ok(_) => (),
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbCommentRemoveErr::Db(err));
                }
            }
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_comment_remove() {
    // use crate::create_user_id;

    init_log();
    let db = Db::test_db(0, "test_comment_remove").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token, 10, 10)
        .await
        .unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite2.token, 10, 10)
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

    let result = db.comment_remove(0, user.username.clone(), 0).await;
    assert!(matches!(
        result,
        Err(DbCommentRemoveErr::CommentNotFound(_))
    ));

    // let result = db.comment_remove(0, "invalid", comment1.id).await;
    // assert!(matches!(result, Err(DbCommentRemoveErr::UserNotFound(_))));

    let result = db
        .comment_remove(0, user2.username.clone(), comment1.id)
        .await;
    assert!(matches!(result, Err(DbCommentRemoveErr::Unauthorized)));

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 1);

    db.comment_remove(0, user.username.clone(), comment1.id)
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 0);

    let comment2 = db
        .comment_add(2, user.username.clone(), post1.id, None, "one2")
        .await
        .unwrap();

    let comment3 = db
        .comment_add(
            3,
            user.username.clone(),
            post1.id,
            Some(comment2.id),
            "one3",
        )
        .await
        .unwrap();

    let comment4 = db
        .comment_add(
            4,
            user2.username.clone(),
            post1.id,
            Some(comment3.id),
            "one4",
        )
        .await
        .unwrap();

    let comment5 = db
        .comment_add(5, user.username.clone(), post1.id, None, "one5")
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 4);
    assert_eq!(comments[3].text, "one2");
    assert_eq!(comments[3].replies_count, 1);
    assert_eq!(comments[3].parents.len(), 0);
    assert_eq!(comments[2].text, "one3");
    assert_eq!(comments[2].replies_count, 1);
    assert_eq!(comments[2].parents.len(), 1);
    assert_eq!(comments[2].parents[0], comments[3].id);
    assert_eq!(comments[1].text, "one4");
    assert_eq!(comments[1].replies_count, 0);
    assert_eq!(comments[1].parents.len(), 2);
    assert_eq!(comments[1].parents[0], comments[3].id);
    assert_eq!(comments[1].parents[1], comments[2].id);
    assert_eq!(comments[0].text, "one5");
    assert_eq!(comments[0].replies_count, 0);
    assert_eq!(comments[0].parents.len(), 0);

    db.comment_remove(0, user.username.clone(), comment3.id)
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[1].text, "one2");
    assert_eq!(comments[1].replies_count, 0);
    assert_eq!(comments[1].parents.len(), 0);
    assert_eq!(comments[0].text, "one5");
    assert_eq!(comments[0].replies_count, 0);
    assert_eq!(comments[0].parents.len(), 0);
}
