use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbComment, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_comment_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbCommentUpdateTextErr {
    #[error("comment not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn comment_update_text(
        &self,
        time: u128,
        user_id: RecordId,
        comment_key: impl Into<RecordIdKey>,
        new_text: impl Into<String>,
    ) -> Result<DbComment, DbCommentUpdateTextErr> {
        let comment_id = create_comment_id(comment_key);

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $comment = SELECT user FROM ONLY $comment_id;

                    IF !$comment {
                        THROW "not found"
                    };

                    IF $comment.user != $user_id {
                        THROW "unauthorized"
                    };

                    UPDATE ONLY $comment_id SET text = $new_text, modified_at = $time RETURN *, user.*;

                    COMMIT TRANSACTION;
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("comment_id", comment_id))
            .bind(("new_text", new_text.into()))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbCommentUpdateTextErr::NotFound,
                err if err.thrown("unauthorized") => DbCommentUpdateTextErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbCommentUpdateTextErr::Db(err)
                }
            })
            .and_then_take_expect(4)
    }
}

#[tokio::test]
async fn test_comment_update_text() {
    // use crate::create_user_id;
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
        .post_add(0, user.id.clone(), "title", "description", "tags")
        .await
        .unwrap();

    let comment1 = db
        .comment_add(
            0,
            user.id.clone(),
            post1.id.key.clone(),
            None::<RecordIdKey>,
            "one",
        )
        .await
        .unwrap();

    assert_eq!(comment1.text, "one");

    let comment1 = db
        .comment_update_text(0, user.id.clone(), comment1.id.key.clone(), "one1")
        .await
        .unwrap();
    assert_eq!(comment1.text, "one1");

    let result = db
        .comment_update_text(0, user.id.clone(), "invalid", "one1")
        .await;
    assert!(matches!(result, Err(DbCommentUpdateTextErr::NotFound)));

    let result = db
        .comment_update_text(0, user2.id.clone(), comment1.id.key.clone(), "one1")
        .await;
    assert!(matches!(result, Err(DbCommentUpdateTextErr::Unauthorized)));
}
