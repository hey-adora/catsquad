use crate::{Db, DbFileImage};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbFileImageGetByHashErr {
    #[error("not found")]
    NotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn file_image_get_by_hash(
        &self,
        img_hash: i64,
    ) -> Result<DbFileImage, DbFileImageGetByHashErr> {
        let pool = &self.db;

        let query = "SELECT * FROM files_images WHERE image_hash = $1";

        let result = sqlx::query_as(query).bind(img_hash).fetch_one(pool).await;

        debug!("query {query}\nresults {result:?}");

        let img = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbFileImageGetByHashErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbFileImageGetByHashErr::Db(err));
            }
        };

        Ok(img)
    }
}
