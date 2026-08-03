use catsquad_log::prelude::*;
use catsquad_shared::PostState;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbPost, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostUpdateStateErr {
    #[error("same state")]
    SameState,

    #[error("cant set draft")]
    CantSetDraft,

    #[error("not active")]
    PostNotActive,

    // #[error("already active")]
    // PostAlreadyActive,
    #[error("post not found")]
    PostNotFound,

    #[error("user not found")]
    UserNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_update_state(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
        state: PostState,
    ) -> Result<DbPost, DbPostUpdateStateErr> {
        let post_id = create_post_id(post_key);
        // TODO add auth check

        let query = r#"
                    BEGIN TRANSACTION;

                    IF !$user_id.exists() {
                        THROW "user not found"
                    };

                    LET $post = SELECT user, state FROM ONLY $post_id;

                    IF $new_state == $draft_state {
                        THROW "cant set draft"
                    };

                    IF $new_state == $post.state {
                        THROW "same stage"
                    };

                    IF $new_state != $active_state && $post.state == $draft_state {
                        THROW "not active"
                    };

                    IF !$post {
                        THROW "not found"
                    };

                    IF $post.user != $user_id {
                        THROW "unauthorized"
                    };

                    UPDATE $post_id SET
                        state = $new_state,
                        modified_at = $time
                    RETURN *, user.*;

                    COMMIT TRANSACTION;
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("draft_state", PostState::Draft.to_string()))
            .bind(("active_state", PostState::Active.to_string()))
            .bind(("hidden_state", PostState::Hidden.to_string()))
            .bind(("post_id", post_id))
            .bind(("user_id", user_id))
            .bind(("new_state", state.to_string()))
            .await
            .check_better(|err| match err {
                err if err.thrown("cant set draft") => DbPostUpdateStateErr::CantSetDraft,
                err if err.thrown("same stage") => DbPostUpdateStateErr::SameState,
                // err if err.thrown("already active") => DbPostUpdateStateErr::PostAlreadyActive,
                err if err.thrown("not active") => DbPostUpdateStateErr::PostNotActive,
                err if err.thrown("user not found") => DbPostUpdateStateErr::UserNotFound,
                err if err.thrown("not found") => DbPostUpdateStateErr::PostNotFound,
                err if err.thrown("unauthorized") => DbPostUpdateStateErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbPostUpdateStateErr::Db(err)
                }
            })
            .and_then_take_expect(8)
    }
}

#[tokio::test]
async fn test_post_update_state() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let post = db
        .post_add(0, user.id.clone(), "title", "description", "")
        .await
        .unwrap();
    assert_eq!(post.state, PostState::Draft.to_string());

    let result = db
        .post_update_state(0, user.id.clone(), post.id.key.clone(), PostState::Draft)
        .await;
    assert_eq!(result, Err(DbPostUpdateStateErr::CantSetDraft));

    let result = db
        .post_update_state(0, user.id.clone(), post.id.key.clone(), PostState::Hidden)
        .await;
    assert_eq!(result, Err(DbPostUpdateStateErr::PostNotActive));

    let post = db
        .post_update_state(0, user.id.clone(), post.id.key.clone(), PostState::Active)
        .await
        .unwrap();
    assert_eq!(post.state, PostState::Active.to_string());

    let post = db
        .post_update_state(0, user.id.clone(), post.id.key.clone(), PostState::Hidden)
        .await
        .unwrap();

    let result = db
        .post_update_state(0, user.id.clone(), post.id.key.clone(), PostState::Draft)
        .await;
    assert_eq!(result, Err(DbPostUpdateStateErr::CantSetDraft));

    let result = db
        .post_update_state(0, user.id.clone(), post.id.key.clone(), PostState::Hidden)
        .await;
    assert_eq!(result, Err(DbPostUpdateStateErr::SameState));

    let _post = db
        .post_update_state(0, user.id.clone(), post.id.key.clone(), PostState::Active)
        .await
        .unwrap();
}
