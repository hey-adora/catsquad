use catsquad_log::prelude::*;
use catsquad_shared::PostState;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbPost, DbPostFile, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostUpdateFileRemoveErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("file already exists")]
    FileNotFound,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_update_file_remove(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
        file_hash: impl Into<String>,
    ) -> Result<DbPostFile, DbPostUpdateFileRemoveErr> {
        let file_hash = file_hash.into();
        let post_id = create_post_id(post_key);

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $post = SELECT file, size_bytes, user FROM ONLY $post_id;

                    IF !$post {
                        THROW "not found"
                    };

                    IF $post.user != $user_id {
                        THROW "unauthorized"
                    };

                    LET $file_index = $post.file.find_index(|$v| $v.hash == $file_hash);

                    IF $file_index == NONE {
                        THROW "file not found"
                    };

                    $post.file[$file_index];

                    LET $file_size_bytes = $post.file[$file_index].size_bytes;

                    UPDATE $user_id SET 
                       used_storage_bytes -= $file_size_bytes, 
                       modified_at = $time
                    RETURN NONE;

                    UPDATE ONLY post SET 
                       file = $post.file.remove($file_index), 
                       size_bytes -= $file_size_bytes, 
                       modified_at = $time 
                    WHERE id = $post_id AND user = $user_id
                    RETURN NONE;

                    COMMIT TRANSACTION;
                    
                "#;

        // SELECT *, user.* FROM $post_id;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("file_hash", file_hash.clone()))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbPostUpdateFileRemoveErr::PostNotFound,
                err if err.thrown("file not found") => DbPostUpdateFileRemoveErr::FileNotFound,
                err if err.thrown("unauthorized") => DbPostUpdateFileRemoveErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbPostUpdateFileRemoveErr::Db(err)
                }
            })
            .and_then_take_expect(6)
    }
}

#[tokio::test]
async fn test_post_update_file_remove() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 25, 15)
        .await
        .unwrap();

    let invite1 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user.id.clone(), "title1", "description1", "tags")
        .await
        .unwrap();
    let post1 = db
        .post_update_state(0, user.id.clone(), post1.id.key, PostState::Active)
        .await
        .unwrap();
    assert_eq!(post1.file.len(), 0);
    assert_eq!(post1.size_bytes, 0);
    assert_eq!(post1.user.used_storage_bytes, 0);

    let post1 = db
        .post_update_file_add(
            0,
            user.id.clone(),
            post1.id.key.clone(),
            10,
            "hash1",
            "png",
            10,
            10,
        )
        .await
        .unwrap();

    assert_eq!(post1.file.len(), 1);
    assert_eq!(post1.file[0].hash, "hash1");
    assert_eq!(post1.size_bytes, 10);
    assert_eq!(post1.user.used_storage_bytes, 10);

    let post1 = db
        .post_update_file_add(
            0,
            user.id.clone(),
            post1.id.key.clone(),
            15,
            "hash2",
            "png",
            10,
            10,
        )
        .await
        .unwrap();

    assert_eq!(post1.file.len(), 2);
    assert_eq!(post1.file[0].hash, "hash1");
    assert_eq!(post1.file[1].hash, "hash2");
    assert_eq!(post1.size_bytes, 25);
    assert_eq!(post1.user.used_storage_bytes, 25);

    let result = db
        .post_update_file_remove(0, user.id.clone(), "invalid", "invalid")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateFileRemoveErr::PostNotFound)
    ));

    let result = db
        .post_update_file_remove(0, user2.id.clone(), post1.id.key.clone(), "invalid")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateFileRemoveErr::Unauthorized)
    ));

    let result = db
        .post_update_file_remove(0, user.id.clone(), post1.id.key.clone(), "invalid")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateFileRemoveErr::FileNotFound)
    ));

    let post1_file = db
        .post_update_file_remove(0, user.id.clone(), post1.id.key.clone(), "hash2")
        .await
        .unwrap();
    let post1 = db.post_get_by_key(post1.id.key.clone()).await.unwrap();

    assert_eq!(post1.file.len(), 1);
    assert_eq!(post1.file[0].hash, "hash1");
    assert_eq!(post1.size_bytes, 10);
    assert_eq!(post1.user.used_storage_bytes, 10);

    let result = db
        .post_update_file_remove(0, user.id.clone(), post1.id.key.clone(), "hash2")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateFileRemoveErr::FileNotFound)
    ));

    //
    let post1_file = db
        .post_update_file_remove(0, user.id.clone(), post1.id.key.clone(), "hash1")
        .await
        .unwrap();
    let post1 = db.post_get_by_key(post1.id.key.clone()).await.unwrap();

    assert_eq!(post1.file.len(), 0);
    assert_eq!(post1.size_bytes, 0);
    assert_eq!(post1.user.used_storage_bytes, 0);

    let result = db
        .post_update_file_remove(0, user.id.clone(), post1.id.key.clone(), "hash1")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateFileRemoveErr::FileNotFound)
    ));
}
