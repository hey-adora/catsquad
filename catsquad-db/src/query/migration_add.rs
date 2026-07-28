use crate::{Db, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};
use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbMigration {
    pub id: RecordId,
    pub version: u64,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbMigrationAddErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),

    #[error("migration \"{0}\" already eixsts")]
    AlreadyExists(u64),

    #[error("migration not found")]
    NotFound,
}

impl Db {
    pub async fn migration_define(&self) {
        let query = "
            DEFINE TABLE migration SCHEMAFULL;
            DEFINE FIELD version ON TABLE migration TYPE int;
            DEFINE FIELD modified_at ON TABLE migration TYPE number;
            DEFINE FIELD created_at ON TABLE migration TYPE number;
            DEFINE INDEX idx_migration_version ON TABLE migration COLUMNS version UNIQUE;
        ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn migration_add(
        &self,
        time: u128,
        version: u64,
    ) -> Result<DbMigration, DbMigrationAddErr> {
        let query = "
            CREATE migration SET version = $version, modified_at = $time, created_at = $time;
        ";
        trace!("about to run {query}");
        self.db
            .query(query)
            .bind(("time", time))
            .bind(("version", version))
            .await
            .check_good(|err| match err {
                err if err.index_exists("idx_migration_version") => {
                    DbMigrationAddErr::AlreadyExists(version)
                }
                err => {
                    error!("unexpected db error {err}");
                    DbMigrationAddErr::from(err)
                }
            })
            .and_then_take_or(0, DbMigrationAddErr::NotFound)
    }
}

#[tokio::test]
async fn test_migration_add() {
    init_log();
    let db = Db::mem(0).await;

    db.migration_add(0, 9999).await.unwrap();
    let result = db.migration_add(0, 9999).await;
    assert_eq!(result, Err(DbMigrationAddErr::AlreadyExists(9999)));
}
