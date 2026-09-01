use crate::Db;
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetPasswordErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_get_password(
        &self,
        email: impl Into<String>,
    ) -> Result<String, DbUserGetPasswordErr> {
        let pool = &self.db;
        let email = email.into();

        let query = "SELECT user_password FROM users WHERE user_email = $1";

        debug!("about to run {query}");

        let result = sqlx::query_as(query).bind(email).fetch_one(pool).await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbUserGetPasswordErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserGetPasswordErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");
        let (user_password,): (String,) = result;

        Ok(user_password)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_get_password() {
    init_log();

    let db = Db::test_db(0, "test_user_get_password").await;

    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    db.user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
        .await
        .unwrap();
    let password = db.user_get_password("hey@hey.com").await.unwrap();
    assert_eq!(password, "hey");
}
