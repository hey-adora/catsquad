use catsquad_log::prelude::*;
use catsquad_shared::PostState;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{Db, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbPostLike {
    pub id: RecordId,
    pub user: RecordId,
    pub post: RecordId,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostLikeAddErr {
    #[error("cant like your own post")]
    CantLikeYourself,

    #[error("post was already liked")]
    PostWasAlreadyLiked,

    #[error("post \"{0}\" was not found")]
    PostNotFound(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

pub fn create_post_like_id(key: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("post", key.into())
}

impl Db {
    pub async fn post_like_define(&self) {
        let query = "
                DEFINE TABLE post_like SCHEMAFULL;
                DEFINE FIELD user ON TABLE post_like TYPE record<user>;
                DEFINE FIELD post ON TABLE post_like TYPE record<post>;
                DEFINE FIELD modified_at ON TABLE post_like TYPE number;
                DEFINE FIELD created_at ON TABLE post_like TYPE number;
                DEFINE INDEX idx_user_post ON TABLE post_like COLUMNS user, post UNIQUE;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn post_like_add(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
    ) -> Result<DbPostLike, DbPostLikeAddErr> {
        // TODO probably add check if user is not banned or something
        let post_key = post_key.into();
        let post_id = create_post_id(post_key.clone());

        let query = r#"
                 BEGIN TRANSACTION;

                 LET $post = SELECT id, user, state FROM ONLY $post_id;

                 IF $post.user == $user_id {
                     THROW "your_post"
                 };

                 IF $post.state == $draft_state || $post.state == $hidden_state {
                     THROW "unauthorized"
                 };

                 CREATE post_like SET
                    user = $user_id,
                    post = $post.id,
                    modified_at = $time,
                    created_at = $time
                 RETURN *;

                 COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");
        self.db
            .query(query)
            .bind(("time", time))
            .bind(("draft_state", PostState::Draft.to_string()))
            .bind(("hidden_state", PostState::Hidden.to_string()))
            // .bind(("active_state", PostState::Active.to_string()))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("your_post") => DbPostLikeAddErr::CantLikeYourself,
                err if err.thrown("unauthorized") => DbPostLikeAddErr::Unauthorized,
                err if err.index_exists("idx_user_post") => DbPostLikeAddErr::PostWasAlreadyLiked,
                err if err.field_value_null("post") => {
                    DbPostLikeAddErr::PostNotFound(post_key.to_sql())
                }
                err => {
                    error!("unexpected db error {err}");
                    DbPostLikeAddErr::Db(err)
                }
            })
            .and_then_take_expect(4)
    }
}

#[tokio::test]
async fn test_post_like_add() {
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
        .post_add(0, user1.id.clone(), "title", "description", "tags")
        .await
        .unwrap();

    let result = db
        .post_like_add(0, user1.id.clone(), post1.id.key.clone())
        .await;
    assert!(matches!(result, Err(DbPostLikeAddErr::CantLikeYourself)));

    let result = db
        .post_like_add(0, user2.id.clone(), post1.id.key.clone())
        .await;
    assert!(matches!(result, Err(DbPostLikeAddErr::Unauthorized)));

    let result = db.post_like_add(0, user1.id.clone(), "wtf").await;
    assert!(matches!(result, Err(DbPostLikeAddErr::PostNotFound(_))));

    db.post_update_state(0, user1.id.clone(), post1.id.key.clone(), PostState::Active)
        .await
        .unwrap();

    let _post_like = db
        .post_like_add(0, user2.id.clone(), post1.id.key.clone())
        .await
        .unwrap();

    let result = db
        .post_like_add(0, user2.id.clone(), post1.id.key.clone())
        .await;
    assert!(matches!(result, Err(DbPostLikeAddErr::PostWasAlreadyLiked)));

    db.post_update_state(0, user1.id.clone(), post1.id.key.clone(), PostState::Hidden)
        .await
        .unwrap();

    let result = db
        .post_like_add(0, user2.id.clone(), post1.id.key.clone())
        .await;
    assert!(matches!(result, Err(DbPostLikeAddErr::Unauthorized)));
}
