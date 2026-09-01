use crate::{Db, DbUser, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdatePasswordByEmailErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_update_password_by_email(
        &self,
        time: u64,
        email: impl Into<String>,
        new_password: impl Into<String>,
    ) -> Result<(), DbUserUpdatePasswordByEmailErr> {
        let pool = &self.db;

        let query =
            "UPDATE users SET user_password = $1, user_modified_at = $2 WHERE user_email = $3";

        debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(new_password.into())
            .bind(XTimestamp(time as i64))
            .bind(email.into())
            .execute(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbUserUpdatePasswordByEmailErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserUpdatePasswordByEmailErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_update_password_by_email() {
    init_log();

    let db = Db::test_db(0, "test_user_update_password_by_email").await;

    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
        .await
        .unwrap();
    assert_eq!(user.password, "hey");
    db.user_update_password_by_email(0, "hey@hey.com", "hey2")
        .await
        .unwrap();
    let user = db.user_get_by_email("hey@hey.com").await.unwrap();
    assert_eq!(user.password, "hey2");
}
