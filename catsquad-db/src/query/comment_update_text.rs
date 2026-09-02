use crate::{Db, DbComment, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::PostState;

#[derive(Debug, thiserror::Error)]
pub enum DbCommentUpdateTextErr {
    #[error("comment not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn comment_update_text(
        &self,
        time: u64,
        user_username: impl Into<String>,
        comment_id: i64,
        new_text: impl Into<String>,
    ) -> Result<(), DbCommentUpdateTextErr> {
        let user_username = user_username.into();
        let new_text = new_text.into();
        let pool = &self.db;

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        // get post
        {
            let query = "SELECT
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

            let (commet_user_username, post_user_username, post_state): (String, String, String) =
                match result {
                    Ok(v) => v,
                    Err(sqlx::Error::RowNotFound) => {
                        return Err(DbCommentUpdateTextErr::NotFound);
                    }
                    Err(err) => {
                        error!("unexpected db error {err}");
                        return Err(DbCommentUpdateTextErr::Db(err));
                    }
                };

            let post_state = PostState::from(post_state);

            if user_username != commet_user_username {
                return Err(DbCommentUpdateTextErr::Unauthorized);
            }

            let post_state = PostState::from(post_state);
            match post_state {
                PostState::Hidden if post_user_username == user_username => (),
                PostState::Active => (),
                _ => return Err(DbCommentUpdateTextErr::Unauthorized),
            }
        }

        // update comment
        {
            let query = "UPDATE comments SET
                            comment_text = $3,
                            comment_modified_at = $1
                            WHERE comment_id = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(comment_id)
                .bind(new_text)
                .execute(&mut *tx)
                .await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbCommentUpdateTextErr::Db(err));
                }
            };

            assert_eq!(result.rows_affected(), 1);
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("post_like_add {err}"))?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_comment_update_text() {
    // use crate::create_user_id;
    init_log();

    let db = Db::test_db(0, "test_comment_update_text").await;

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

    let _comment2 = db
        .comment_add(1, user.username.clone(), post1.id, None, "one2")
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].text, "one2");
    assert_eq!(comments[1].text, "one");

    db.comment_update_text(0, user.username.clone(), comment1.id, "one1")
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].text, "one2");
    assert_eq!(comments[1].text, "one1");

    let result = db
        .comment_update_text(0, user.username.clone(), 0, "one1")
        .await;
    assert!(matches!(result, Err(DbCommentUpdateTextErr::NotFound)));

    let result = db
        .comment_update_text(0, user2.username.clone(), comment1.id, "one1")
        .await;
    assert!(matches!(result, Err(DbCommentUpdateTextErr::Unauthorized)));
}
