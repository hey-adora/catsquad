use catsquad_log::prelude::*;

use crate::{Db, DbComment, SurrealCheckUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbCommentGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn comment_get_all(&self) -> Result<Vec<DbComment>, DbCommentGetAllErr> {
        let query = "SELECT *, user.* FROM comment ORDER BY created_at DESC;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbCommentGetAllErr::Db(err)
                }
            })
            .and_then_take_all(0)
    }
}

// test is in /query/post_comment_add.rs
