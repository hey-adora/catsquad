use catsquad_log::prelude::*;
use catsquad_shared::PostState;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{
    Db, DbInvite, DbPost, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostGetByKeyErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_get_by_key(
        &self,
        user_id: Option<RecordId>,
        post_key: impl Into<RecordIdKey>,
    ) -> Result<DbPost, DbPostGetByKeyErr> {
        let post_id = create_post_id(post_key);
        let query = r#"
                BEGIN TRANSACTION;

                let $post = SELECT *, user.* FROM ONLY $post_id;

                IF $post.user.id != $user_id && $post.state == $draft_state {
                    THROW "unauthorized"
                };

                RETURN $post;

                COMMIT TRANSACTION;
            "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("draft_state", PostState::Draft.to_string()))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("unauthorized") => DbPostGetByKeyErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbPostGetByKeyErr::Db(err)
                }
            })
            .and_then_take_or(3, DbPostGetByKeyErr::PostNotFound)
    }
}

#[tokio::test]
async fn test_post_get_by_key() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user1 = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite2.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user1.id.clone(), "title1", "description1", "tags")
        .await
        .unwrap();

    let post2 = db
        .post_get_by_key(Some(user1.id.clone()), post1.id.key.clone())
        .await
        .unwrap();
    assert_eq!(post1.id, post2.id);

    let result = db.post_get_by_key(None, post1.id.key.clone()).await;
    assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized)));

    let result = db
        .post_get_by_key(Some(user2.id.clone()), post1.id.key.clone())
        .await;
    assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized)));

    db.post_update_state(0, user1.id.clone(), post1.id.key.clone(), PostState::Active)
        .await
        .unwrap();

    let result = db.post_get_by_key(None, post1.id.key.clone()).await;
    assert!(matches!(result, Ok(_)));
}
