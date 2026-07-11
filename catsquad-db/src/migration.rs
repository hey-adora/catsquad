use crate::{Db, query::migration_get_latest::DbMigrationGetLatestErr};
use catsquad_log::prelude::*;

pub async fn migrate(db: &Db) {
    let latest = db.migration_get_latest().await;

    match latest {
        Ok(migration) if migration.version == 0 => {
            info!("database has latest migration");
        }
        Err(DbMigrationGetLatestErr::NotFound) => {
            db.migration_define().await;
            db.user_define().await;
            db.invite_define().await;
            db.email_sent_define().await;
            info!("database migration successful");
        }
        Ok(migration) => panic!("migration unsupported version {}", migration.version),
        Err(DbMigrationGetLatestErr::Db(_)) => panic!("migration failed"),
    }
}

#[tokio::test]
async fn test_migrate() {
    init_log();
    let db = Db::mem().await;
}
