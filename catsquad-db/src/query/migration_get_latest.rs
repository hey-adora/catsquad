use crate::{Db, DbMigration, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbMigrationGetLatestErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),

    #[error("migration not found")]
    NotFound,
}

impl Db {
    pub async fn migration_get_latest(&self) -> Result<DbMigration, DbMigrationGetLatestErr> {
        let query = "
            SELECT * FROM ONLY migration ORDER BY created_at DESC
        ";
        trace!("about to run {query}");
        //The table 'migration' does not exist
        self.db
            .query(query)
            .await
            .check_good(|err| match err {
                err if err.table_not_found("migration") => DbMigrationGetLatestErr::NotFound,
                err => {
                    error!("unexpected db error {err}");
                    DbMigrationGetLatestErr::from(err)
                }
            })
            .and_then_take_or(0, DbMigrationGetLatestErr::NotFound)
    }
}
