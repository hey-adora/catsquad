use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbPostLike, SurrealCheckUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostLikeGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_like_get_all(&self) -> Result<Vec<DbPostLike>, DbPostLikeGetAllErr> {
        let query = "SELECT * FROM post_like ORDER BY created_at DESC;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbPostLikeGetAllErr::Db(err)
                }
            })
            .and_then_take_all(0)
    }
}

#[tokio::test]
async fn test_post_like_get_all() {
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
    let _post_like = db
        .post_like_add(0, user.id.clone(), post1.id.key.clone())
        .await
        .unwrap();
    let post_likes = db.post_like_get_all().await.unwrap();
    assert_eq!(post_likes.len(), 1);
}
