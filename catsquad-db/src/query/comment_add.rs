use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_invite_id,
    create_post_id, create_user_id,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbComment {
    pub id: RecordId,
    pub user: DbUser,
    pub post: RecordId,
    pub replies_count: usize,
    pub parent: Vec<RecordId>,
    pub text: String,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbCommentAddErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),

    #[error("post \"{0}\" was not found")]
    PostNotFound(String),

    #[error("user \"{0}\" was not found")]
    UserNotFound(String),

    #[error("reply_comment \"{0}\" was not found")]
    ParentNotFound(String),
}

pub fn create_comment_id(key: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("comment", key.into())
}

impl Db {
    pub async fn comment_define(&self) {
        let query = "
                DEFINE TABLE comment SCHEMAFULL;
                DEFINE FIELD user ON TABLE comment TYPE record<user>;
                DEFINE FIELD post ON TABLE comment TYPE record<post>;
                DEFINE FIELD parent ON TABLE comment TYPE array<record<comment>>;
                DEFINE FIELD replies_count ON TABLE comment TYPE number;
                DEFINE FIELD text ON TABLE comment TYPE string;
                DEFINE FIELD modified_at ON TABLE comment TYPE number;
                DEFINE FIELD created_at ON TABLE comment TYPE number;
                DEFINE INDEX idx_comment_parent ON TABLE comment COLUMNS parent;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn comment_add(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
        comment_parent_key: Option<impl Into<RecordIdKey>>,
        text: impl Into<String>,
    ) -> Result<DbComment, DbCommentAddErr> {
        // TODO check if user exists in other queries too
        let post_key = post_key.into();
        let post_id = create_post_id(post_key.clone());
        let parent_id = comment_parent_key.map(|v| create_comment_id(v));

        // IF $post.user != $user_id {
        //     THROW "unauthorized"
        // };

        let query = r#"
                 BEGIN TRANSACTION;

                 LET $post = SELECT NONE FROM ONLY $post_id;
                 LET $user = SELECT NONE FROM ONLY $user_id;

                 IF !$user {
                     THROW "user not found"
                 };

                 IF !$post {
                     THROW "not found"
                 };
                 
                 LET $parent = IF $parent_id {
                         SELECT id, parent, replies_count FROM ONLY $parent_id                 
                     } ELSE {
                         NULL
                     };

                 IF $parent_id AND !$parent {
                     THROW "parent not found"
                 };

                 IF $parent {
                    UPDATE $parent.id SET replies_count = $parent.replies_count + 1;
                 };

                 LET $parent = if $parent {
                        if $parent.parent { $parent.parent } else { [] } + [$parent.id]
                    } else {
                        []
                    };

                 CREATE comment SET
                    user = $user_id,
                    post = $post_id,
                    parent = $parent,
                    replies_count = 0,
                    text = $comment_text,
                    modified_at = $time,
                    created_at = $time
                 RETURN *, user.*;

                 COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");
        self.db
            .query(query)
            .bind(("time", time))
            .bind(("user_id", user_id.clone()))
            .bind(("post_id", post_id.clone()))
            .bind(("comment_text", text.into()))
            .bind(("parent_id", parent_id.clone()))
            .await
            .check_better(|err| match err {
                err if err.thrown("not found") => DbCommentAddErr::PostNotFound(post_key.to_sql()),
                err if err.thrown("user not found") => {
                    DbCommentAddErr::UserNotFound(user_id.key.to_sql())
                }
                err if err.thrown("parent not found") => DbCommentAddErr::ParentNotFound(
                    parent_id
                        .map(|v| v.key.to_sql())
                        .unwrap_or_else(|| "invalid".to_string()),
                ),
                err => {
                    error!("unexpected db error {err}");
                    DbCommentAddErr::Db(err)
                }
            })
            .and_then_take_expect(9)
    }
}

#[tokio::test]
async fn test_comment_add() {
    use crate::create_user_id;

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

    let result = db
        .comment_add(0, user.id.clone(), "invalid", None::<RecordIdKey>, "one1")
        .await;
    assert!(matches!(result, Err(DbCommentAddErr::PostNotFound(_))));

    let result = db
        .comment_add(
            0,
            create_user_id("invalid"),
            post1.id.key.clone(),
            None::<RecordIdKey>,
            "one2",
        )
        .await;
    assert!(matches!(result, Err(DbCommentAddErr::UserNotFound(_))));

    let result = db
        .comment_add(
            0,
            user.id.clone(),
            post1.id.key.clone(),
            Some("invalid"),
            "one2",
        )
        .await;
    assert!(matches!(result, Err(DbCommentAddErr::ParentNotFound(_))));

    let _comment2 = db
        .comment_add(
            1,
            user.id.clone(),
            post1.id.key.clone(),
            Some(comment1.id.key.clone()),
            "one3",
        )
        .await
        .unwrap();

    let comments = db.comment_get_all().await.unwrap();

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].parent.len(), 1);
    assert_eq!(comments[0].parent[0], comment1.id.clone());
    assert_eq!(comments[0].text, "one3");
    assert_eq!(comments[1].parent.len(), 0);
    assert_eq!(comments[1].text, "one");
}
