use crate::{Db, DbMigration, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbMigrationGetLatestErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("migration not found")]
    NotFound,
}

impl Db {
    pub async fn migration_get_latest(&self) -> Result<DbMigration, DbMigrationGetLatestErr> {
        let pool = &self.db;

        let mut tx = pool.begin().await?;

        let query = "SELECT * FROM migrations ORDER BY migration_created_at DESC LIMIT 1";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_one(&mut *tx).await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbMigrationGetLatestErr::NotFound);
            }
            Err(sqlx::Error::Database(err))
                if err.kind() == sqlx::error::ErrorKind::Other
                    && err.message() == "relation \"migrations\" does not exist" =>
            {
                return Err(DbMigrationGetLatestErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbMigrationGetLatestErr::Db(err));
            }
        };

        let (version, XTimestamp(modified_at), XTimestamp(created_at)): (i32, XTimestamp, XTimestamp) =
            result;

        Ok(DbMigration {
            version: version as u16,
            modified_at: modified_at as u64,
            created_at: created_at as u64,
        })
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_migration_get_latest() {
    init_log();
    let db = Db::test_db(0, "test_migration_get_latest").await;

    let _result = db.migration_get_latest().await.unwrap();
}
