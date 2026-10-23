use crate::{Db, query::role_add::DbRole};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbRoleGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn role_get_all(&self) -> Result<Vec<DbRole>, DbRoleGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM roles ORDER BY role_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let sessions = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbRoleGetAllErr::Db(err));
            }
        };

        Ok(sessions)
    }
}
