use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{
    Db, DbInvite, DbPost, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostGetByKeyErr {
    #[error("post not found")]
    PostNotFound,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_get_by_key(
        &self,
        post_key: impl Into<RecordIdKey>,
    ) -> Result<DbPost, DbPostGetByKeyErr> {
        let post_id = create_post_id(post_key);
        let query = "SELECT *, user.* FROM ONLY $post_id;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("post_id", post_id))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbPostGetByKeyErr::Db(err)
                }
            })
            .and_then_take_or(0, DbPostGetByKeyErr::PostNotFound)
    }
}

#[tokio::test]
async fn test_post_get_by_key() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();

    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user.id.clone(), "title1", "description1", "tags")
        .await
        .unwrap();

    let post2 = db.post_get_by_key(post1.id.key.clone()).await.unwrap();
    assert_eq!(post1.id, post2.id);
}
