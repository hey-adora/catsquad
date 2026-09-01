use crate::{Db, DbPasswordChange, DbUser};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserPasswordChangeGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn password_change_get_all(
        &self,
    ) -> Result<Vec<DbPasswordChange>, DbUserPasswordChangeGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM passwords_changes ORDER BY password_change_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserPasswordChangeGetAllErr::Db(err));
            }
        };

        Ok(result)
        // let query = "SELECT *, user.* FROM password_change ORDER BY created_at DESC;";

        // trace!("about to run {query}");

        // self.db
        //     .query(query)
        //     .await
        //     .check_good(|err| match err {
        //         err => {
        //             error!("unexpected db error {err}");
        //             DbUserPasswordChangeGetAllErr::DB(err)
        //         }
        //     })
        //     .and_then_take_all(0)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_password_change_get_all() {
    init_log();

    let db = Db::test_db(0, "test_password_change_get_all").await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    db.user_add(0, "hey", "hey", invite.token, 10, 10)
        .await
        .unwrap();
    let users = db.password_change_add(0, "hey@hey.com", 10).await.unwrap();
    let password_changes = db.password_change_get_all().await.unwrap();
    assert_eq!(password_changes.len(), 1);
}
