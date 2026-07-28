use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbPost, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostUpdateDescriptionErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_update_description(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
        new_description: impl Into<String>,
    ) -> Result<DbPost, DbPostUpdateDescriptionErr> {
        let post_id = create_post_id(post_key);

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $post = SELECT user FROM ONLY $post_id;

                    IF !$post {
                        THROW "not found"
                    };

                    IF $post.user != $user_id {
                        THROW "unauthorized"
                    };

                    UPDATE ONLY $post_id SET description = $new_description, modified_at = $time RETURN *, user.*;

                    COMMIT TRANSACTION;
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .bind(("new_description", new_description.into()))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbPostUpdateDescriptionErr::PostNotFound,
                err if err.thrown("unauthorized") => DbPostUpdateDescriptionErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbPostUpdateDescriptionErr::Db(err)
                }
            })
            .and_then_take_expect(4)
    }
}

#[tokio::test]
async fn test_post_update_description() {
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
    assert_eq!(post1.description, "description1");

    let post1 = db
        .post_update_description(0, user.id.clone(), post1.id.key.clone(), "description2")
        .await
        .unwrap();
    assert_eq!(post1.description, "description2");

    let result = db
        .post_update_description(0, user2.id.clone(), post1.id.key.clone(), "description2")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateDescriptionErr::Unauthorized)
    ));

    let result = db
        .post_update_description(0, user.id.clone(), "invalid", "description2")
        .await;
    assert!(matches!(
        result,
        Err(DbPostUpdateDescriptionErr::PostNotFound)
    ));
}
