use crate::{Db, DbFileImage};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbFileImageGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn file_image_get_all(&self) -> Result<Vec<DbFileImage>, DbFileImageGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM files_images ORDER BY image_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let posts = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbFileImageGetAllErr::Db(err));
            }
        };

        Ok(posts)
    }
}
