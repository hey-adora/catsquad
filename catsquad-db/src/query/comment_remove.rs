use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbComment, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    create_comment_id, create_invite_id, create_post_id, create_user_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbCommentRemoveErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),

    #[error("comment \"{0}\" was not found")]
    NotFound(String),

    #[error("user \"{0}\" was not found")]
    UserNotFound(String),

    #[error("unauthorized")]
    Unauthorized,
}

impl Db {
    pub async fn comment_remove(
        &self,
        time: u128,
        user_id: RecordId,
        comment_key: impl Into<RecordIdKey>,
    ) -> Result<(), DbCommentRemoveErr> {
        let comment_key = comment_key.into();
        let comment_id = create_comment_id(comment_key.clone());

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $comment_user = SELECT id FROM ONLY $user_id;
                    if !$comment_user {
                        THROW "user not found";
                    };

                    LET $comment = SELECT id, parent, user, replies_count FROM ONLY $comment_id;

                    IF !$comment {
                        THROW "not found"
                    };

                    IF $comment.user != $user_id {
                        THROW "unauthorized"
                    };

                    LET $last = $comment.parent.last();
                    LET $parent = IF $last {
                        SELECT id, replies_count FROM ONLY $last
                    } ELSE {
                        NULL
                    };
                    if $parent.replies_count > 0 {
                        UPDATE $parent.id SET replies_count -= 1, modified_at = $time;
                    };

                    DELETE comment WHERE parent.find($comment_id);
                    DELETE ONLY $comment_id;

                    COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");
        self.db
            .query(query)
            .bind(("time", time))
            .bind(("comment_id", comment_id))
            .bind(("user_id", user_id.clone()))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => {
                    DbCommentRemoveErr::NotFound(comment_key.to_sql())
                }
                err if err.thrown("user not found") => {
                    DbCommentRemoveErr::UserNotFound(user_id.key.to_sql())
                }
                err if err.thrown("unauthorized") => DbCommentRemoveErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbCommentRemoveErr::Db(err)
                }
            })
            .map(|_| ())
        // .and_then_take_expect(10)
    }
}

#[tokio::test]
async fn test_comment_remove() {
    use crate::create_user_id;

    init_log();
    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite2.id.key.clone(), 10, 10)
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

    let result = db.comment_remove(0, user.id.clone(), "invalid").await;
    assert!(matches!(result, Err(DbCommentRemoveErr::NotFound(_))));

    let result = db
        .comment_remove(0, create_user_id("invalid"), comment1.id.key.clone())
        .await;
    assert!(matches!(result, Err(DbCommentRemoveErr::UserNotFound(_))));

    let result = db
        .comment_remove(0, user2.id.clone(), comment1.id.key.clone())
        .await;
    assert!(matches!(result, Err(DbCommentRemoveErr::Unauthorized)));

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 1);

    db.comment_remove(0, user.id.clone(), comment1.id.key.clone())
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 0);

    let comment2 = db
        .comment_add(
            2,
            user.id.clone(),
            post1.id.key.clone(),
            None::<RecordIdKey>,
            "one2",
        )
        .await
        .unwrap();

    let comment3 = db
        .comment_add(
            3,
            user.id.clone(),
            post1.id.key.clone(),
            Some(comment2.id.key.clone()),
            "one3",
        )
        .await
        .unwrap();

    let comment4 = db
        .comment_add(
            4,
            user2.id.clone(),
            post1.id.key.clone(),
            Some(comment3.id.key.clone()),
            "one4",
        )
        .await
        .unwrap();

    let comment5 = db
        .comment_add(
            5,
            user.id.clone(),
            post1.id.key.clone(),
            None::<RecordIdKey>,
            "one5",
        )
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 4);
    assert_eq!(comments[3].text, "one2");
    assert_eq!(comments[3].replies_count, 1);
    assert_eq!(comments[3].parent.len(), 0);
    assert_eq!(comments[2].text, "one3");
    assert_eq!(comments[2].replies_count, 1);
    assert_eq!(comments[2].parent.len(), 1);
    assert_eq!(comments[2].parent[0], comments[3].id);
    assert_eq!(comments[1].text, "one4");
    assert_eq!(comments[1].replies_count, 0);
    assert_eq!(comments[1].parent.len(), 2);
    assert_eq!(comments[1].parent[0], comments[3].id);
    assert_eq!(comments[1].parent[1], comments[2].id);
    assert_eq!(comments[0].text, "one5");
    assert_eq!(comments[0].replies_count, 0);
    assert_eq!(comments[0].parent.len(), 0);

    db.comment_remove(0, user.id.clone(), comment3.id.key.clone())
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[1].text, "one2");
    assert_eq!(comments[1].replies_count, 0);
    assert_eq!(comments[1].parent.len(), 0);
    assert_eq!(comments[0].text, "one5");
    assert_eq!(comments[0].replies_count, 0);
    assert_eq!(comments[0].parent.len(), 0);
}
