use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdatePasswordByIdErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_update_password_by_username(
        &self,
        time: u64,
        username: impl Into<String>,
        new_password: impl Into<String>,
    ) -> Result<(), DbUserUpdatePasswordByIdErr> {
        let pool = &self.db;
        let username = username.into();
        let new_password = new_password.into();
        let time = XTimestamp(time as i64);

        let query =
            "UPDATE users SET user_password = $1, user_modified_at = $2 WHERE user_username = $3";

        debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(new_password)
            .bind(time)
            .bind(username)
            .execute(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbUserUpdatePasswordByIdErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserUpdatePasswordByIdErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_update_password_by_username() {
    init_log();

    let db = Db::test_db(0, "test_user_update_password_by_id").await;

    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
        .await
        .unwrap();
    assert_eq!(user.password, "hey");
    db.user_update_password_by_username(0, user.username.clone(), "hey2")
        .await
        .unwrap();
    let user = db.user_get_by_username("hey").await.unwrap();
    assert_eq!(user.password, "hey2");
}
