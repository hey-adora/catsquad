use crate::{Db, DbUser};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetByUsernameErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_get_by_username(
        &self,
        username: impl Into<String>,
    ) -> Result<DbUser, DbUserGetByUsernameErr> {
        let pool = &self.db;
        let username = username.into();

        let query = "SELECT * FROM users WHERE user_username = $1";

        debug!("about to run {query}");

        let result = sqlx::query_as(query).bind(username).fetch_one(pool).await;

        let users = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbUserGetByUsernameErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserGetByUsernameErr::Db(err));
            }
        };

        debug!("query {query}\nresults {users:?}");

        Ok(users)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_get_by_username() {
    init_log();

    let db = Db::test_db(0, "test_user_get_by_username").await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let invite2 = db.invite_add(0, "hey2@hey.com", 10).await.unwrap();

    db.user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
        .await
        .unwrap();
    db.user_add(0, "hey2", "hey", invite2.token.clone(), 10, 10)
        .await
        .unwrap();
    let user = db.user_get_by_username("hey2").await.unwrap();
    assert_eq!(user.username, "hey2");
    let user = db.user_get_by_username("hey3").await;
    assert!(matches!(user, Err(DbUserGetByUsernameErr::NotFound)));
}
