use catsquad_log::prelude::*;

use crate::{Db, DbUser};

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn user_get_all(&self) -> Result<Vec<DbUser>, DbUserGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM users ORDER BY user_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let users = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserGetAllErr::Db(err));
            }
        };

        Ok(users)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_get_all() {
    init_log();

    let db = Db::test_db(0, "test_user_get_all").await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    db.user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
        .await
        .unwrap();
    let users = db.user_get_all().await.unwrap();
    assert_eq!(users.len(), 1);
}
