use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DbMigration {
    pub version: u16,
    pub modified_at: u64,
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbMigrationAddErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("migration \"{0}\" already eixsts")]
    AlreadyExists(u16),
}

impl Db {
    pub async fn migration_define(&self) {
        let pool = &self.db;
        let _result = sqlx::raw_sql(
            "
            CREATE TABLE migrations (
                migration_version int PRIMARY KEY,
                migration_modified_at timestamp NOT NULL,
                migration_created_at timestamp NOT NULL
            );
        ",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    pub async fn migration_add(
        &self,
        time: u64,
        version: u16,
    ) -> Result<DbMigration, DbMigrationAddErr> {
        let pool = &self.db;

        let result = sqlx::query(
            "
            INSERT INTO migrations (
                    migration_version,
                    migration_modified_at,
                    migration_created_at
                )
                VALUES ( $1, $2, $2 );
        ",
        )
        .bind(version as i32)
        .bind(XTimestamp(time as i64))
        .execute(pool)
        .await;

        let _result = match result {
            Ok(v) => v,
            Err(sqlx::Error::Database(err))
                if err.is_unique_violation() && err.constraint() == Some("migrations_pkey") =>
            {
                return Err(DbMigrationAddErr::AlreadyExists(version));
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbMigrationAddErr::Db(err));
            }
        };

        Ok(DbMigration {
            version,
            modified_at: time,
            created_at: time,
        })
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_migration_add() {
    init_log();
    let db = Db::test_db(0, "test_migration_add").await;

    db.migration_add(0, 9999).await.unwrap();
    let result = db.migration_add(0, 9999).await;
    assert!(matches!(result, Err(DbMigrationAddErr::AlreadyExists(_))));
}
