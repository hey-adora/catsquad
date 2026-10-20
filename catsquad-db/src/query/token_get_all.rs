use crate::{Db, DbToken, DbUser};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbTokenGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn token_get_all(&self) -> Result<Vec<DbToken>, DbTokenGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM tokens ORDER BY token_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbTokenGetAllErr::Db(err));
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_token_get_all() {
    use crate::DbTokenLabel;

    init_log();

    let db = Db::test_db(0, "test_token_get_all").await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    db.user_add(0, "hey", "hey", invite.token, 10, 10)
        .await
        .unwrap();
    db.token_add(0, "hey", 10, DbTokenLabel::PasswordChange)
        .await
        .unwrap();
    let tokens = db.token_get_all().await.unwrap();
    assert_eq!(tokens.len(), 1);
}
