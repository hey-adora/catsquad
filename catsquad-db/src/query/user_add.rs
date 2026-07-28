use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_invite_id};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DbUser {
    pub id: RecordId,
    pub used_storage_bytes: u64,
    pub max_storage_per_file_bytes: u64,
    pub max_storage_bytes: u64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbUserAddErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),

    #[error("email is taken")]
    EmailIsTaken,

    #[error("username is taken")]
    UsernameIsTaken,

    #[error("invite not found")]
    InviteNotFound,

    #[error("invite already used")]
    InviteAlreadyUsed,

    #[error("invite expired")]
    InviteExpired,
}

pub fn create_user_id(key: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("user", key.into())
}

impl Db {
    pub async fn user_define(&self) {
        let query = "
                DEFINE TABLE user SCHEMAFULL;
                DEFINE FIELD username ON TABLE user TYPE string;
                DEFINE FIELD used_storage_bytes ON TABLE user TYPE number;
                DEFINE FIELD max_storage_per_file_bytes ON TABLE user TYPE number;
                DEFINE FIELD max_storage_bytes ON TABLE user TYPE number;
                DEFINE FIELD email ON TABLE user TYPE string;
                DEFINE FIELD password ON TABLE user TYPE string;
                DEFINE FIELD modified_at ON TABLE user TYPE number;
                DEFINE FIELD created_at ON TABLE user TYPE number;
                DEFINE INDEX idx_user_username ON TABLE user COLUMNS username UNIQUE;
                DEFINE INDEX idx_user_email ON TABLE user COLUMNS email UNIQUE;
            ";
        trace!("about to run {query}");
        self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn user_add(
        &self,
        time: u128,
        username: impl Into<String>,
        password: impl Into<String>,
        invite_token: impl Into<RecordIdKey>,
        max_storage: u64,
        max_storage_per_file: u64,
    ) -> Result<DbUser, DbUserAddErr> {
        let username = username.into();
        let password = password.into();
        let invite_token = create_invite_id(invite_token.into());

        let query = r#"
                 BEGIN TRANSACTION;
                 LET $invite = SELECT * FROM ONLY $invite_token;
                 if !$invite {
                     THROW "invite not found"
                 };
                 if $invite.used {
                     THROW "invite already used"
                 };
                 if $invite.expires < $time {
                     THROW "invite expired"
                 };
                 CREATE user SET
                    username = $username,
                    email = $invite.email,
                    used_storage_bytes = 0,
                    max_storage_per_file_bytes = $max_storage_per_file,
                    max_storage_bytes = $max_storage,
                    password = $password,
                    modified_at = $time,
                    created_at = $time;
                UPDATE $invite_token SET used = true;
                COMMIT TRANSACTION;
                "#;
        trace!("about to run {query}");
        self.db
            .query(query)
            .bind(("max_storage", max_storage))
            .bind(("max_storage_per_file", max_storage_per_file))
            .bind(("time", time))
            .bind(("username", username.clone()))
            // .bind(("email", email.clone()))
            .bind(("password", password))
            .bind(("invite_token", invite_token))
            .await
            .check_better(|err| match err {
                err if err.thrown("invite not found") => DbUserAddErr::InviteNotFound,
                err if err.thrown("invite already used") => DbUserAddErr::InviteAlreadyUsed,
                err if err.thrown("invite expired") => DbUserAddErr::InviteExpired,
                err if err.index_exists("idx_user_email") => DbUserAddErr::EmailIsTaken,
                err if err.index_exists("idx_user_username") => DbUserAddErr::UsernameIsTaken,
                err => {
                    error!("unexpected db error {err}");
                    DbUserAddErr::Db(err)
                }
            })
            .and_then_take_expect(5)
    }
}

#[tokio::test]
async fn test_user_add() {
    init_log();
    let db = Db::mem(0).await;

    let result = db.user_add(0, "hey", "hey", "invalid", 10, 10).await;
    assert_eq!(result, Err(DbUserAddErr::InviteNotFound));

    let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    let result = db.user_add(2, "hey", "hey", invite.id.key, 10, 10).await;
    assert_eq!(result, Err(DbUserAddErr::InviteExpired));

    let invite = db.invite_add(2, "hey@hey.com", 3).await.unwrap();
    let result = db
        .user_add(3, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await;
    assert!(result.is_ok());

    let result = db
        .user_add(2, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await;
    assert_eq!(result, Err(DbUserAddErr::InviteAlreadyUsed));

    let invite = db.invite_add(2, "hey2@hey.com", 3).await.unwrap();
    let result = db
        .user_add(2, "hey", "hey", invite.id.key.clone(), 10, 10)
        .await;
    assert_eq!(result, Err(DbUserAddErr::UsernameIsTaken));
}
