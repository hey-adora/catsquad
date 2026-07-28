use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbUser, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbPost {
    pub id: RecordId,
    pub user: DbUser,
    pub show: bool,
    pub title: String,
    pub tags: String,
    pub description: String,
    pub favorites: u64,
    pub size_bytes: usize,
    pub file: Vec<DbPostFile>,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbPostFile {
    pub proccesed: bool,
    pub extension: String,
    pub hash: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostAddErr {
    #[error("user not found")]
    UserNotFound,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

pub fn create_post_id(key: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("post", key.into())
}

impl Db {
    pub async fn post_define(&self) {
        let query = "
                DEFINE TABLE post SCHEMAFULL;
                DEFINE FIELD user ON TABLE post TYPE record<user>;
                DEFINE FIELD show ON TABLE post TYPE bool;
                DEFINE FIELD title ON TABLE post TYPE string;
                DEFINE FIELD size_bytes ON TABLE post TYPE number;
                DEFINE FIELD description ON TABLE post TYPE string;
                DEFINE FIELD tags ON TABLE post TYPE string;
                DEFINE FIELD favorites ON TABLE post TYPE number;
                DEFINE FIELD file ON TABLE post TYPE array<object>;
                DEFINE FIELD file.*.proccesed ON TABLE post TYPE bool;
                DEFINE FIELD file.*.extension ON TABLE post TYPE string;
                DEFINE FIELD file.*.hash ON TABLE post TYPE string;
                DEFINE FIELD file.*.size_bytes ON TABLE post TYPE int;
                DEFINE FIELD file.*.width ON TABLE post TYPE int;
                DEFINE FIELD file.*.height ON TABLE post TYPE int;
                DEFINE FIELD modified_at ON TABLE post TYPE number;
                DEFINE FIELD created_at ON TABLE post TYPE number;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn post_add(
        &self,
        time: u128,
        user_id: RecordId,
        title: impl Into<String>,
        description: impl Into<String>,
        tags: impl Into<String>,
    ) -> Result<DbPost, DbPostAddErr> {
        // let user_id = create_user_id(user_key);
        let title = title.into();
        let description = description.into();
        let tags = tags.into();
        // let favorites = favorites.into();

        let query = r#"
                 BEGIN TRANSACTION;
                 LET $user = SELECT id FROM ONLY $user_id;
                 CREATE post SET
                    user = $user.id,
                    show = true,
                    title = $title,
                    description = $description,
                    tags = $tags,
                    size_bytes = 0,
                    favorites = 0,
                    file = [],
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
            .bind(("title", title))
            .bind(("description", description))
            .bind(("tags", tags))
            .await
            .check_better(|err| match err {
                err if err.field_value_null("user") => DbPostAddErr::UserNotFound,
                err => {
                    error!("unexpected db error {err}");
                    DbPostAddErr::Db(err)
                }
            })
            .and_then_take_expect(2)
    }
}

#[tokio::test]
async fn test_post_add() {
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

    assert_eq!(post1.title, "title");
    assert_eq!(post1.description, "description");
    assert_eq!(post1.tags, "tags");

    let result = db
        .post_add(0, create_user_id("invalid"), "title", "description", "tags")
        .await;
    assert_eq!(result, Err(DbPostAddErr::UserNotFound));
}
