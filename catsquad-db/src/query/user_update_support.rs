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
    use catsquad_shared::SessionTokenKind;

    use crate::user_add_for_testing;

    init_log();

    let db = Db::test_db(0, "test_user_update_support").await;

    let user = user_add_for_testing(&db, 0, "hey", "pss", "hey@heyadora.com", 10, 10).await;
    assert_eq!(user.support, "");

    db.user_update_support(0, user.username.clone(), "one two three")
        .await
        .unwrap();

    let user = db
        .user_get_by_username(SessionTokenKind::None, "hey")
        .await
        .unwrap();

    assert_eq!(user.support, "one two three");
}
