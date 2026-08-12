use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbPost, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id,
    proccess_tags,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostUpdateTagsErr {
    #[error("user not found")]
    UserNotFound,

    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_update_tags(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
        new_tags: impl Into<String>,
    ) -> Result<DbPost, DbPostUpdateTagsErr> {
        let post_id = create_post_id(post_key);
        let tags = proccess_tags(new_tags);

        let query = r#"
                    BEGIN TRANSACTION;

                    IF !$user_id.exists() {
                        THROW "user not found"
                    };

                    LET $post = SELECT user FROM ONLY $post_id;

                    IF !$post {
                        THROW "not found"
                    };

                    IF $post.user != $user_id {
                        THROW "unauthorized"
                    };

                    UPDATE ONLY $post_id SET tags = $new_tags, modified_at = $time RETURN *, user.*;

                    COMMIT TRANSACTION;
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .bind(("new_tags", tags))
            .await
            .check_better(|err| match err {
                err if err.thrown("user not found") => DbPostUpdateTagsErr::UserNotFound,
                err if err.thrown("not found") => DbPostUpdateTagsErr::PostNotFound,
                err if err.thrown("unauthorized") => DbPostUpdateTagsErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbPostUpdateTagsErr::Db(err)
                }
            })
            .and_then_take_expect(5)
    }
}

#[tokio::test]
async fn test_post_update_tags() {
    use crate::create_user_id;
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

    let post1 = db
        .post_add(0, user.id.clone(), "title1", "description1", "tags")
        .await
        .unwrap();
    assert_eq!(post1.title, "title1");

    let post1 = db
        .post_update_tags(0, user.id.clone(), post1.id.key.clone(), " tags2   Tag3")
        .await
        .unwrap();
    assert_eq!(post1.tags, " tags2 tag3 ");

    let result = db
        .post_update_tags(0, create_user_id("invalid"), post1.id.key.clone(), "tags2")
        .await;
    assert!(matches!(result, Err(DbPostUpdateTagsErr::UserNotFound)));

    let result = db
        .post_update_tags(0, user2.id.clone(), post1.id.key.clone(), "tags2")
        .await;
    assert!(matches!(result, Err(DbPostUpdateTagsErr::Unauthorized)));

    let result = db
        .post_update_tags(0, user.id.clone(), "invalid", "tags2")
        .await;
    assert!(matches!(result, Err(DbPostUpdateTagsErr::PostNotFound)));
}
