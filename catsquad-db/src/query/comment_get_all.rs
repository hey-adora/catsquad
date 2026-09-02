use crate::{Db, DbComment};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbCommentGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn comment_get_all(&self) -> Result<Vec<DbComment>, DbCommentGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM comments ORDER BY comment_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let posts = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbCommentGetAllErr::Db(err));
            }
        };

        Ok(posts)
    }
}

// test is in /query/post_comment_add.rs
