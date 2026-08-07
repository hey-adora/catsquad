use catsquad_log::prelude::*;
use catsquad_shared::PostState;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbPostLike, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostLikeRemoveErr {
    #[error("post not found")]
    PostNotFound,

    #[error("not found")]
    LikeNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_like_remove(
        &self,
        time: u128,
        user_id: RecordId,
        post_key: impl Into<RecordIdKey>,
    ) -> Result<DbPostLike, DbPostLikeRemoveErr> {
        let post_key = post_key.into();
        let post_id = create_post_id(post_key.clone());

        let query = r#"
                    BEGIN TRANSACTION;

                    LET $post = SELECT user, state FROM ONLY $post_id;

                    IF !$post {
                        THROW "post_not_found"
                    };

                    IF $post.state == $draft_state || $post.state == $hidden_state {
                        THROW "unauthorized"
                    };

                    DELETE post_like WHERE
                        user = $user_id AND
                        post = $post_id
                    RETURN BEFORE;

                    COMMIT TRANSACTION;
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("draft_state", PostState::Draft.to_string()))
            .bind(("hidden_state", PostState::Hidden.to_string()))
            .bind(("user_id", user_id))
            .bind(("post_id", post_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("post_not_found") => DbPostLikeRemoveErr::PostNotFound,
                err if err.thrown("unauthorized") => DbPostLikeRemoveErr::Unauthorized,
                err => {
                    error!("unexpected db error {err}");
                    DbPostLikeRemoveErr::Db(err)
                }
            })
            .and_then_take_or(4, DbPostLikeRemoveErr::LikeNotFound)
    }
}

#[tokio::test]
async fn test_post_like_remove() {
    init_log();
    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user1 = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();
    let invite2 = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
    let user2 = db
        .user_add(0, "hey2", "hey", invite2.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user1.id.clone(), "title", "description", "tags")
        .await
        .unwrap();

    {
        let result = db.post_like_remove(0, user1.id.clone(), "invalid").await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::PostNotFound)));

        let result = db
            .post_like_remove(0, user1.id.clone(), post1.id.key.clone())
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::Unauthorized)));

        let result = db
            .post_like_remove(0, user2.id.clone(), post1.id.key.clone())
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::Unauthorized)));
    }

    db.post_update_state(
        0,
        user1.id.clone(),
        post1.id.key.clone(),
        catsquad_shared::PostState::Active,
    )
    .await
    .unwrap();

    {
        let result = db
            .post_like_remove(0, user1.id.clone(), post1.id.key.clone())
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::LikeNotFound)));

        let result = db
            .post_like_remove(0, user2.id.clone(), post1.id.key.clone())
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::LikeNotFound)));

        let _post_like = db
            .post_like_add(0, user2.id.clone(), post1.id.key.clone())
            .await
            .unwrap();

        let _post_like = db
            .post_like_remove(0, user2.id.clone(), post1.id.key.clone())
            .await
            .unwrap();
    }

    db.post_update_state(
        0,
        user1.id.clone(),
        post1.id.key.clone(),
        catsquad_shared::PostState::Hidden,
    )
    .await
    .unwrap();

    {
        let result = db
            .post_like_remove(0, user1.id.clone(), post1.id.key.clone())
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::Unauthorized)));

        let result = db
            .post_like_remove(0, user2.id.clone(), post1.id.key.clone())
            .await;
        assert!(matches!(result, Err(DbPostLikeRemoveErr::Unauthorized)));
    }
}
