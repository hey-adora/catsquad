use crate::{Db, DbComment};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbCommentGetByIdErr {
    #[error("not found")]
    NotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    /// dont use in api, it has no permission checks
    pub async fn comment_get_by_id(
        &self,
        comment_id: i64,
    ) -> Result<DbComment, DbCommentGetByIdErr> {
        let pool = &self.db;
        let query = "SELECT * FROM comments WHERE comment_id = $1";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).bind(comment_id).fetch_one(pool).await;

        let comment = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbCommentGetByIdErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbCommentGetByIdErr::Db(err));
            }
        };

        Ok(comment)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_comment_get_by_id() {
    use catsquad_shared::PostState;

    init_log();
    let db = Db::test_db(0, "test_comment_get_by_id").await;

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

    let result = db.comment_get_by_id(comment1.id).await.unwrap();
    assert_eq!(comment1.id, result.id);
}
