use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbPost, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostUpdateProccesedErr {
    #[error("not found")]
    NotFound,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_update_proccsed(
        &self,
        time: u128,
        post_key: impl Into<RecordIdKey>,
        file_hash: impl Into<String>,
    ) -> Result<DbPost, DbPostUpdateProccesedErr> {
        // TODO make query simpler
        let post_id = create_post_id(post_key);

        let query = r#"
                    UPDATE $post_id SET file = file.map(|$v| {
                      IF $v.hash = $file_hash {
                          {
                            proccesed: true,
                            extension: $v.extension,
                            hash: $v.hash,
                            size_bytes: $v.size_bytes,
                            width: $v.width,
                            height: $v.height,
                          }
                       } ELSE { $v }
                    }), modified_at = $time RETURN *, user.*;
                    
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("post_id", post_id))
            .bind(("file_hash", file_hash.into()))
            .await
            .check_better(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbPostUpdateProccesedErr::Db(err)
                }
            })
            .and_then_take_or(0, DbPostUpdateProccesedErr::NotFound)
    }
}

#[tokio::test]
async fn test_post_update_proccesed() {
    use crate::{DbPost, DbUser};

    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let invite1 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let post = db
        .post_add(0, user.id.clone(), "title", "description", "")
        .await
        .unwrap();
    let post2 = db
        .post_add(1, user2.id.clone(), "title2", "description", "")
        .await
        .unwrap();

    assert!(post.file.len() == 0);

    let add_post_file_fn = async |user: &DbUser, post: &DbPost, hash: &str, size: u64| {
        db.post_update_file_add(
            2,
            user.id.clone(),
            post.id.key.clone(),
            size,
            hash,
            "png",
            50,
            50,
        )
        .await
    };
    let post = add_post_file_fn(&user, &post, "1", 1).await.unwrap();
    let post = add_post_file_fn(&user, &post, "2", 1).await.unwrap();
    let post2 = add_post_file_fn(&user2, &post2, "1", 1).await.unwrap();
    let post = db
        .post_update_proccsed(0, post.id.key.clone(), "1")
        .await
        .unwrap();

    let posts = db.post_get_unproccesed().await.unwrap();
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].file.len(), 2);
    assert_eq!(posts[0].file[0].hash, "1");
    assert_eq!(posts[0].file[0].proccesed, true);
    assert_eq!(posts[0].file[1].hash, "2");
    assert_eq!(posts[0].file[1].proccesed, false);
    assert_eq!(posts[1].file.len(), 1);
    assert_eq!(posts[1].file[0].hash, "1");
    assert_eq!(posts[1].file[0].proccesed, false);
}
