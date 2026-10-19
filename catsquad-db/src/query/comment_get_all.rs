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
        let query = "SELECT
                                comment_id,
                                comment_user_username,
                                user_pfp_image_hash as comment_user_pfp_hash,
                                coalesce(image_processed, false) as comment_user_pfp_proccesed,
                                comment_post_id,
                                comment_replies_count,
                                comment_user_username,
                                comment_parent_id,
                                comment_parents,
                                comment_text,
                                comment_modified_at,
                                comment_created_at
                            FROM comments
                            LEFT JOIN users ON comment_user_username=user_username
                            LEFT JOIN files_images ON user_pfp_image_hash=image_hash
                            ORDER BY comment_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let comments = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbCommentGetAllErr::Db(err));
            }
        };

        Ok(comments)
    }
}

// test is in /query/post_comment_add.rs
