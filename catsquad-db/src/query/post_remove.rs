use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbComment, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    create_comment_id, create_invite_id, create_post_id, create_user_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostRemoveErr {
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
    pub async fn post_remove(
        &self,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
    ) -> Result<(), DbPostRemoveErr> {
        let post_key = post_key.into();
        let post_id = create_post_id(post_key.clone());

        let query = r#"
                 BEGIN TRANSACTION;

                 IF !$user_id.exists() {
                     THROW "user not found";
                 };

                 LET $post = SELECT user FROM ONLY $post_id;

                 IF !$post {
                     THROW "post not found";
                 };

                 IF $post.user != $user_id {
                     THROW "unauthorized"
                 };

                 DELETE $post_id;
                 DELETE comment WHERE post == $post_id;
                 DELETE post_like WHERE post = $post_id;

                 COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");
        self.db
            .query(query)
            .bind(("post_id", post_id))
            .bind(("user_id", user_id.clone()))
            .await
            .check_better(|err| match err {
                err if err.thrown("post not found") => DbPostRemoveErr::NotFound(post_key.to_sql()),
                err if err.thrown("user not found") => {
                    DbPostRemoveErr::UserNotFound(user_id.key.to_sql())
                }
                err if err.thrown("unauthorized") => DbPostRemoveErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbPostRemoveErr::Db(err)
                }
            })
            .map(|_| ())
    }
}

#[tokio::test]
async fn test_post_remove() {
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

    let result = db.post_remove(user.id.clone(), "invalid").await;
    assert!(matches!(result, Err(DbPostRemoveErr::NotFound(_))));

    let post1_key = {
        let post1 = db
            .post_add(0, user.id.clone(), "title", "description", "tags")
            .await
            .unwrap();

        db.post_like_add(0, user2.id.clone(), post1.id.key.clone())
            .await
            .unwrap();

        let comment1 = db
            .comment_add(
                0,
                user2.id.clone(),
                post1.id.key.clone(),
                None::<RecordIdKey>,
                "one",
            )
            .await
            .unwrap();

        db.comment_add(
            0,
            user2.id.clone(),
            post1.id.key.clone(),
            Some(comment1.id.key.clone()),
            "one1",
        )
        .await
        .unwrap();

        post1.id.key
    };

    let post2_key = {
        let post2 = db
            .post_add(0, user.id.clone(), "title2", "description", "tags")
            .await
            .unwrap();

        db.post_like_add(0, user.id.clone(), post2.id.key.clone())
            .await
            .unwrap();

        db.comment_add(
            0,
            user.id.clone(),
            post2.id.key.clone(),
            None::<RecordIdKey>,
            "one2",
        )
        .await
        .unwrap();
        post2.id.key
    };

    let result = db.post_remove(user2.id.clone(), post1_key.clone()).await;
    assert!(matches!(result, Err(DbPostRemoveErr::Unauthorized)));

    let result = db
        .post_remove(create_user_id("invalid"), post1_key.clone())
        .await;
    assert!(matches!(result, Err(DbPostRemoveErr::UserNotFound(_))));

    let posts = db.post_get_all().await.unwrap();
    let comments = db.comment_get_all().await.unwrap();
    let likes = db.post_like_get_all().await.unwrap();

    assert_eq!(posts.len(), 2);
    assert_eq!(comments.len(), 3);
    assert_eq!(likes.len(), 2);

    db.post_remove(user.id.clone(), post1_key.clone())
        .await
        .unwrap();

    let posts = db.post_get_all().await.unwrap();
    let comments = db.comment_get_all().await.unwrap();
    let likes = db.post_like_get_all().await.unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(comments.len(), 1);
    assert_eq!(likes.len(), 1);
    assert_eq!(posts[0].title, "title2");
    assert_eq!(comments[0].text, "one2");
    assert_eq!(likes[0].post.key, post2_key);
}
