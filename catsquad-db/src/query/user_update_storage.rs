use catsquad_log::prelude::*;
use catsquad_shared::{MAX_STORAGE, MAX_STORAGE_PER_FILE};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{
    Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_user_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserUpdateStorageErr {
    #[error("user not found")]
    UserNotFound,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn user_update_storage(
        &self,
        time: u128,
        user_id: RecordId,
        max_storage_bytes: u64,
        max_storage_per_file_bytes: u64,
    ) -> Result<DbUser, DbUserUpdateStorageErr> {
        let query = r#"
                 UPDATE $user_id SET
                         max_storage_per_file_bytes = $max_storage_per_file_bytes,
                         max_storage_bytes = $max_storage_bytes
                    RETURN *;
                "#;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("max_storage_per_file_bytes", max_storage_per_file_bytes))
            .bind(("max_storage_bytes", max_storage_bytes))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbUserUpdateStorageErr::Db(err)
                }
            })
            .and_then_take_or(0, DbUserUpdateStorageErr::UserNotFound)
    }
}

#[tokio::test]
async fn test_user_update_storage() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(
            0,
            "hey",
            "hey",
            invite1.id.key.clone(),
            MAX_STORAGE,
            MAX_STORAGE_PER_FILE,
        )
        .await
        .unwrap();
    assert_eq!(user.max_storage_bytes, MAX_STORAGE);
    assert_eq!(user.max_storage_per_file_bytes, MAX_STORAGE_PER_FILE);

    let user = db
        .user_update_storage(0, user.id.clone(), 10, 5)
        .await
        .unwrap();
    assert_eq!(user.max_storage_bytes, 10);
    assert_eq!(user.max_storage_per_file_bytes, 5);
}
