use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbPost, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostUpdateOrderErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("invalid selected index")]
    InvalidIndex,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_update_order(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
        selected_pos: usize,
        new_pos: usize,
    ) -> Result<DbPost, DbPostUpdateOrderErr> {
        let post_id = create_post_id(post_key);

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $post = SELECT file, user FROM ONLY $post_id;

                    IF $post.user AND $post.user != $user_id {
                        THROW "unauthorized";
                    };

                    IF !$post.file {
                        THROW "not found";
                    };
                    
                    LET $post_files_len = $post.file.len();
                    IF $post_files_len <= $selected_pos OR $post_files_len <= $new_pos {
                        THROW "out of range";
                    };

                    LET $file_selected = $post.file.at($selected_pos);
                    LET $files_removed = $post.file.remove($selected_pos);
                    LET $files_inserted = $files_removed.insert($file_selected, $new_pos);

                    UPDATE ONLY $post_id SET 
                       file = $files_inserted, 
                       modified_at = $time 
                    RETURN *, user.*;

                    COMMIT TRANSACTION;
                    
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("selected_pos", selected_pos))
            .bind(("new_pos", new_pos))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbPostUpdateOrderErr::PostNotFound,
                err if err.thrown("unauthorized") => DbPostUpdateOrderErr::Unauthorized,
                err if err.thrown("out of range") => DbPostUpdateOrderErr::InvalidIndex,
                err => {
                    error!("unexpected db error {err}");
                    DbPostUpdateOrderErr::Db(err)
                }
            })
            .and_then_take_expect(9)
    }
}

#[tokio::test]
async fn test_post_update_order() {
    use crate::DbPost;

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

    let add_post_file_fn = async |post: &DbPost, hash: &str, size: u64| {
        db.post_update_file_add(
            0,
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
    add_post_file_fn(&post, "0", 1).await.unwrap();
    add_post_file_fn(&post, "1", 1).await.unwrap();
    add_post_file_fn(&post, "2", 1).await.unwrap();
    let post = add_post_file_fn(&post, "3", 1).await.unwrap();
    assert_eq!(post.file.len(), 4);
    assert_eq!(post.file[0].hash, "0");
    assert_eq!(post.file[1].hash, "1");
    assert_eq!(post.file[2].hash, "2");
    assert_eq!(post.file[3].hash, "3");

    let result = db
        .post_update_order(0, user2.id.clone(), post.id.key.clone(), 2, 0)
        .await;
    assert!(matches!(result, Err(DbPostUpdateOrderErr::Unauthorized)));

    let post = db
        .post_update_order(0, user.id.clone(), post.id.key.clone(), 2, 0)
        .await
        .unwrap();
    assert_eq!(post.file.len(), 4);
    assert_eq!(post.file[0].hash, "2");
    assert_eq!(post.file[1].hash, "0");
    assert_eq!(post.file[2].hash, "1");
    assert_eq!(post.file[3].hash, "3");

    let post = db
        .post_update_order(0, user.id.clone(), post.id.key.clone(), 0, 2)
        .await
        .unwrap();
    assert_eq!(post.file.len(), 4);
    assert_eq!(post.file[0].hash, "0");
    assert_eq!(post.file[1].hash, "1");
    assert_eq!(post.file[2].hash, "2");
    assert_eq!(post.file[3].hash, "3");

    let post = db
        .post_update_order(0, user.id.clone(), post.id.key.clone(), 0, 3)
        .await
        .unwrap();
    assert_eq!(post.file.len(), 4);
    assert_eq!(post.file[0].hash, "1");
    assert_eq!(post.file[1].hash, "2");
    assert_eq!(post.file[2].hash, "3");
    assert_eq!(post.file[3].hash, "0");

    let post_err = db
        .post_update_order(0, user.id.clone(), post.id.key.clone(), 0, 4)
        .await
        .err()
        .unwrap();
    assert!(matches!(post_err, DbPostUpdateOrderErr::InvalidIndex));

    let post_err = db
        .post_update_order(0, user.id.clone(), post.id.key.clone(), 4, 0)
        .await
        .err()
        .unwrap();
    assert!(matches!(post_err, DbPostUpdateOrderErr::InvalidIndex));

    let post_err = db
        .post_update_order(0, user.id.clone(), "invalid", 4, 0)
        .await
        .err()
        .unwrap();
    assert!(matches!(post_err, DbPostUpdateOrderErr::PostNotFound));
}
