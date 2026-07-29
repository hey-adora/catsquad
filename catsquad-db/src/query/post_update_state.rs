use catsquad_log::prelude::*;
use catsquad_shared::PostState;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{Db, DbPost, SurrealCheckUtils, SurrealSerializeUtils, create_post_id};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostUpdateStateErr {
    #[error("not found")]
    NotFound,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_update_state(
        &self,
        time: u128,
        post_key: impl Into<RecordIdKey>,
        state: PostState,
    ) -> Result<DbPost, DbPostUpdateStateErr> {
        let post_id = create_post_id(post_key);
        // TODO add auth check

        let query = r#"
                    UPDATE $post_id SET
                        state = $post_state,
                        modified_at = $time
                    RETURN *, user.*;
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("post_id", post_id))
            .bind(("post_state", state.to_string()))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbPostUpdateStateErr::Db(err)
                }
            })
            .and_then_take_or(0, DbPostUpdateStateErr::NotFound)
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

    let post = db
        .post_update_state(0, post.id.key, PostState::Active)
        .await
        .unwrap();
    assert_eq!(post.state, PostState::Active.to_string());
}
