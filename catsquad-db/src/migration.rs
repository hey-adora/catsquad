use crate::{Db, DbMigrationGetLatestErr};
use catsquad_log::prelude::*;

impl Db {
    pub async fn migrate(&self, time: u64) {
        let latest = self.migration_get_latest().await;

        match latest {
            Ok(migration) if migration.version == 0 => {
                info!("database has latest migration");
            }
            Err(DbMigrationGetLatestErr::NotFound) => {
                self.migration_define().await;
                self.user_define().await;
                self.invite_define().await;
                self.email_sent_define().await;
                self.session_define().await;
                self.password_change_define().await;
                self.email_change_define().await;
                self.post_define().await;
                self.file_image_define().await;
                self.post_like_define().await;
                self.comment_define().await;

                self.migration_add(time, 0).await.unwrap();
                info!("database migration successful");
            }
            Ok(migration) => panic!("migration unsupported version {}", migration.version),
            Err(DbMigrationGetLatestErr::Db(_)) => panic!("migration failed"),
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_migrate() {
    init_log();
    let _db = Db::test_db(0, "test_migrate").await;
}
