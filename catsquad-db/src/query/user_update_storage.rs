use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdateStorageErr {
    #[error("user not found")]
    UserNotFound,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn user_update_storage(
        &self,
        time: u64,
        user_username: impl Into<String>,
        max_storage_bytes: u32,
        max_storage_per_file_bytes: u32,
    ) -> Result<(), DbUserUpdateStorageErr> {
        let pool = &self.db;
        let user_username = user_username.into();

        let query = "UPDATE users SET
                            user_max_storage_bytes = $3,
                            user_max_storage_per_file_bytes = $4,
                            user_modified_at = $1
                            WHERE user_username = $2";

        debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(XTimestamp(time as i64))
            .bind(user_username)
            .bind(max_storage_bytes as i64)
            .bind(max_storage_per_file_bytes as i64)
            .execute(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbUserUpdateStorageErr::UserNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserUpdateStorageErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_update_storage() {
    init_log();

    let db = Db::test_db(0, "test_user_update_storage").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token.clone(), 10, 5)
        .await
        .unwrap();
    assert_eq!(user.max_storage_bytes, 10);
    assert_eq!(user.max_storage_per_file_bytes, 5);

    db.user_update_storage(0, user.username.clone(), 10, 5)
        .await
        .unwrap();

    let user = db.user_get_by_username("hey").await.unwrap();

    assert_eq!(user.max_storage_bytes, 10);
    assert_eq!(user.max_storage_per_file_bytes, 5);
}
