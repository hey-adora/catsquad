use crate::{Db, DbEmailChange};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbEmailChangeGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_change_get_all(&self) -> Result<Vec<DbEmailChange>, DbEmailChangeGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM emails_changes ORDER BY email_change_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let posts = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbEmailChangeGetAllErr::Db(err));
            }
        };

        Ok(posts)
    }
}
