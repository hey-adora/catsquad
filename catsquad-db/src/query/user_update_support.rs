use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdateSupportErr {
    #[error("user not found")]
    UserNotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn user_update_support(
        &self,
        time: u64,
        user_username: impl Into<String>,
        support: impl Into<String>,
    ) -> Result<(), DbUserUpdateSupportErr> {
        let pool = &self.db;
        let user_username = user_username.into();
        let support = support.into();

        let query = "UPDATE users SET
                            user_support = $3,
                            user_modified_at = $1
                            WHERE user_username = $2";

        debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(XTimestamp(time as i64))
            .bind(user_username)
            .bind(support)
            .execute(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbUserUpdateSupportErr::UserNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserUpdateSupportErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_update_support() {
    init_log();

    let db = Db::test_db(0, "test_user_update_storage").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token.clone(), 10, 5)
        .await
        .unwrap();
    assert_eq!(user.support, "");

    db.user_update_support(0, user.username.clone(), "one two three")
        .await
        .unwrap();

    let user = db.user_get_by_username("hey").await.unwrap();

    assert_eq!(user.support, "one two three");
}
