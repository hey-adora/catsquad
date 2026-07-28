use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{Db, SurrealCheckUtils, SurrealSerializeUtils, create_post_id};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostLikeGetByPostErr {
    #[error("not found")]
    NotFound,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_like_get_by_post(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
    ) -> Result<RecordId, DbPostLikeGetByPostErr> {
        let post_key = post_key.into();
        let post_id = create_post_id(post_key.clone());

        let query = "(SELECT id FROM ONLY post_like WHERE user = $user_id AND post = $post_id).id";

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbPostLikeGetByPostErr::Db(err)
                }
            })
            .and_then_take_or(0, DbPostLikeGetByPostErr::NotFound)
    }
}

#[tokio::test]
async fn test_post_like_get_by_post() {
    init_log();
    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user.id.clone(), "title", "description", "tags")
        .await
        .unwrap();

    let result = db
        .post_like_get_by_post(0, user.id.clone(), post1.id.key.clone())
        .await;
    assert!(matches!(result, Err(DbPostLikeGetByPostErr::NotFound)));

    let _post_like = db
        .post_like_add(0, user.id.clone(), post1.id.key.clone())
        .await
        .unwrap();

    let _post_like_id = db
        .post_like_get_by_post(0, user.id.clone(), post1.id.key.clone())
        .await
        .unwrap();
}
