use crate::Db;
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbRoleRemoveErr {
    #[error("not found")]
    NotFound,

    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn role_remove(&self, role_id: i64) -> Result<(), DbRoleRemoveErr> {
        let pool = &self.db;

        let query = "DELETE FROM roles WHERE role_id = $1";

        let result = sqlx::query(query).bind(role_id).execute(pool).await;

        debug!("query {query}\nresult {result:#?}");

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbRoleRemoveErr::Db(err));
            }
        };

        match result.rows_affected() {
            0 => return Err(DbRoleRemoveErr::NotFound),
            1 => (),
            _ => unreachable!("invalid quary, must only update 1 role max"),
        }

        debug!("query {query}\nresults {result:?}");

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_role_remove() {
    init_log();

    let db = Db::test_db(0, "test_role_remove").await;

    let _role_admin = db.role_add(0, "admin", vec!["ban"]).await.unwrap();
    let role_mod = db.role_add(1, "mod", vec!["kick"]).await.unwrap();

    db.role_remove(role_mod.id).await.unwrap();

    let roles = db.role_get_all().await.unwrap();
    assert_eq!(roles[0].name, "admin");
}
