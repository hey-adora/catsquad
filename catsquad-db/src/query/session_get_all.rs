use crate::{Db, DbSession};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbSessionGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn sessoin_get_all(&self) -> Result<Vec<DbSession>, DbSessionGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM sessions ORDER BY session_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let sessions = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbSessionGetAllErr::Db(err));
            }
        };

        Ok(sessions)
    }
}
