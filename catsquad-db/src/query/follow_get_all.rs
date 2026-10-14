use crate::{Db, query::follow_add::DbFollow};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbFollowGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn follow_get_all(&self) -> Result<Vec<DbFollow>, DbFollowGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM follow ORDER BY follow_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let followers = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbFollowGetAllErr::Db(err));
            }
        };

        Ok(followers)
    }
}
