use crate::{Db, DbConfirmToken, DbUser};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbConfirmTokenGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn confirm_token_get_all(
        &self,
    ) -> Result<Vec<DbConfirmToken>, DbConfirmTokenGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM tokens ORDER BY token_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbConfirmTokenGetAllErr::Db(err));
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_conform_token_get_all() {
    use crate::{DbConfirmTokenLabel, user_add_for_testing};

    init_log();

    let db = Db::test_db(0, "test_token_get_all").await;

    let _user = user_add_for_testing(&db, 0, "hey", "pss", "hey@heyadora.com", 10, 10).await;

    db.confirm_token_add(0, "hey", 10, DbConfirmTokenLabel::PasswordChange)
        .await
        .unwrap();
    let tokens = db.confirm_token_get_all().await.unwrap();
    assert_eq!(tokens.len(), 1);
}
