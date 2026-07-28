use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbPost, DbPostFile, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostUpdateFileAddErr {
    #[error("not enough storage")]
    OutOfStorage,

    #[error("file too big")]
    FileTooBig,

    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("file already exists")]
    FileAlreadyExists,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_update_file_add(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
        file_size: u64,
        file_hash: impl Into<String>,
        file_extension: impl Into<String>,
        file_width: u32,
        file_height: u32,
    ) -> Result<DbPost, DbPostUpdateFileAddErr> {
        // TODO check if max storage is reached
        let file_hash = file_hash.into();
        let post_file = DbPostFile {
            proccesed: false,
            extension: file_extension.into(),
            hash: file_hash.clone(),
            size_bytes: file_size,
            width: file_width,
            height: file_height,
        };
        let post_id = create_post_id(post_key);

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $post = SELECT file, user.id, user.used_storage_bytes, user.max_storage_bytes, user.max_storage_per_file_bytes FROM ONLY $post_id;

                    IF !$post {
                        THROW "not found"
                    };

                    IF $post.user.id != $user_id {
                        THROW "unauthorized"
                    };

                    LET $exists = $post.file.find(|$v| $v.hash = $file_hash);

                    IF $exists {
                        THROW "hash already exists";
                    };

                    IF $post.user.used_storage_bytes + $size_bytes > $post.user.max_storage_bytes  {
                        THROW "out of storage";
                    };

                    IF $size_bytes > $post.user.max_storage_per_file_bytes {
                        THROW "file too big";
                    };
                    
                    UPDATE $user_id SET
                       used_storage_bytes += $size_bytes, 
                       modified_at = $time
                    RETURN NONE;

                    UPDATE ONLY $post_id SET 
                       file += $post_file, 
                       size_bytes += $size_bytes, 
                       modified_at = $time 
                    RETURN *, user.*;

                    COMMIT TRANSACTION;
                    
                "#;

        // SELECT *, user.* FROM $post_id;
        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("file_hash", file_hash.clone()))
            .bind(("size_bytes", file_size))
            .bind(("post_file", post_file))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .bind(("time", time))
            .await
            .check_better(|err| match err {
                err if err.thrown("out of storage") => DbPostUpdateFileAddErr::OutOfStorage,
                err if err.thrown("file too big") => DbPostUpdateFileAddErr::FileTooBig,
                err if err.thrown("not found") => DbPostUpdateFileAddErr::PostNotFound,
                err if err.thrown("hash already exists") => {
                    DbPostUpdateFileAddErr::FileAlreadyExists
                }
                err if err.thrown("unauthorized") => DbPostUpdateFileAddErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbPostUpdateFileAddErr::Db(err)
                }
            })
            .and_then_take_expect(9)
    }
}

#[tokio::test]
async fn test_post_update_file_add() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 9, 5)
        .await
        .unwrap();

    let invite1 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite1.id.key.clone(), 9, 5)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user.id.clone(), "title1", "description1", "tags")
        .await
        .unwrap();
    assert_eq!(post1.file.len(), 0);

    let result = db
        .post_update_file_add(
            0,
            user.id.clone(),
            post1.id.key.clone(),
            6,
            "hash1",
            "png",
            10,
            10,
        )
        .await;
    assert!(matches!(result, Err(DbPostUpdateFileAddErr::FileTooBig)));

    let post1 = db
        .post_update_file_add(
            0,
            user.id.clone(),
            post1.id.key.clone(),
            5,
            "hash1",
            "png",
            10,
            10,
        )
        .await
        .unwrap();

    let result = db
        .post_update_file_add(
            0,
            user.id.clone(),
            post1.id.key.clone(),
            5,
            "hash2",
            "png",
            10,
            10,
        )
        .await;
    assert!(matches!(result, Err(DbPostUpdateFileAddErr::OutOfStorage)));

    let result = db
        .post_update_file_add(
            0,
            user.id.clone(),
            post1.id.key.clone(),
            5,
            "hash1",
            "png",
            10,
            10,
        )
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateFileAddErr::FileAlreadyExists)
    ));

    let result = db
        .post_update_file_add(0, user.id.clone(), "invalid", 5, "hash1", "png", 10, 10)
        .await;
    assert!(matches!(result, Err(DbPostUpdateFileAddErr::PostNotFound)));

    let result = db
        .post_update_file_add(
            0,
            user2.id.clone(),
            post1.id.key.clone(),
            5,
            "hash1",
            "png",
            10,
            10,
        )
        .await;
    assert!(matches!(result, Err(DbPostUpdateFileAddErr::Unauthorized)));
}
