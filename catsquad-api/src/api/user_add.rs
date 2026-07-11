use std::ascii::AsciiExt;

use anyhow::anyhow;
use axum::{
    Form, Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbUser, DbUserAddErr, id_to_string};
use catsquad_log::prelude::*;

use crate::{
    MAX_STORAGE, MAX_STORAGE_PER_FILE,
    state::AppState,
    validation::{validate_password, validate_username},
};

pub const USER_ADD_ENDPOINT: &'static str = "/api/register";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserAddRes {
    pub key: String,
    pub username: String,
    pub email: String,
    pub created_at: u128,
}

impl From<DbUser> for UserAddRes {
    fn from(value: DbUser) -> Self {
        Self {
            key: id_to_string(value.id),
            username: value.username,
            email: value.email,
            created_at: value.created_at,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserAddReq {
    pub username: String,
    pub password: String,
    pub invite_token: String,
}

pub const USER_ADD_REQ_FIELD_USERNAME: &'static str = "username";
pub const USER_ADD_REQ_FIELD_PASSWORD: &'static str = "password";
pub const USER_ADD_REQ_FIELD_INVITE_TOKEN: &'static str = "invite_token";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum UserAddErr {
    #[error("invalid input")]
    InvalidInput {
        username: Option<String>,
        password: Option<String>,
    },

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

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("internal server err")]
    InternalServerErr,
}

impl From<DbUserAddErr> for UserAddErr {
    fn from(value: DbUserAddErr) -> Self {
        match value {
            DbUserAddErr::EmailIsTaken => UserAddErr::EmailIsTaken,
            DbUserAddErr::UsernameIsTaken => UserAddErr::UsernameIsTaken,
            DbUserAddErr::InviteNotFound => UserAddErr::InviteNotFound,
            DbUserAddErr::InviteAlreadyUsed => UserAddErr::InviteAlreadyUsed,
            DbUserAddErr::InviteExpired => UserAddErr::InviteExpired,
            DbUserAddErr::Db(_) => UserAddErr::InternalServerErr,
        }
    }
}

pub fn status_code(result: &Result<UserAddRes, UserAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserAddErr::InvalidInput { .. }) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::EmailIsTaken) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::UsernameIsTaken) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::InviteNotFound) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::InviteAlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::InviteExpired) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_add(
    State(app): State<AppState>,
    Form(req): Form<UserAddReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<UserAddRes, UserAddErr> {
        let username = req.username.trim().to_lowercase();
        let password = req.password;
        let username_err = validate_username(&username);
        let password_err = validate_password(&password);

        if username_err.is_err() || password_err.is_err() {
            return Err(UserAddErr::InvalidInput {
                username: username_err.err(),
                password: password_err.err(),
            });
        }

        let user = app
            .db
            .user_add(
                time,
                username,
                password,
                req.invite_token,
                MAX_STORAGE,
                MAX_STORAGE_PER_FILE,
            )
            .await?;

        let user = UserAddRes::from(user);
        Ok(user)
    };
    let result = inner().await;
    let status_code = status_code(&result);
    (status_code, Json(result))
}

#[cfg(test)]
mod invite_add_test_utils {
    use crate::{
        TestServer,
        api::{
            USER_ADD_ENDPOINT,
            user_add::{
                USER_ADD_REQ_FIELD_INVITE_TOKEN, USER_ADD_REQ_FIELD_PASSWORD,
                USER_ADD_REQ_FIELD_USERNAME, UserAddErr, UserAddRes,
            },
        },
    };

    impl TestServer {
        pub async fn user_add(
            &self,
            username: impl AsRef<str>,
            password: impl AsRef<str>,
            invite_token: impl AsRef<str>,
        ) -> Result<UserAddRes, UserAddErr> {
            let data = format!(
                "{}={}&{}={}&{}={}",
                USER_ADD_REQ_FIELD_USERNAME,
                username.as_ref(),
                USER_ADD_REQ_FIELD_PASSWORD,
                password.as_ref(),
                USER_ADD_REQ_FIELD_INVITE_TOKEN,
                invite_token.as_ref()
            );
            self.post::<Result<UserAddRes, UserAddErr>>(USER_ADD_ENDPOINT, data)
                .await
        }
    }
}

#[tokio::test]
async fn test_user_add() {
    init_log();
    let server = crate::TestServer::new().await;

    let result = server.user_add("hey", "hello", "").await;
    assert!(
        matches!(result, Err(UserAddErr::InvalidInput { username, password }) if username.is_none() && password.is_some())
    );

    let result = server.user_add("he", "hello@P", "").await;
    assert!(
        matches!(result, Err(UserAddErr::InvalidInput { username, password }) if username.is_some() && password.is_some())
    );

    let result = server.user_add("hey", "hello1111111@1P", "invalid").await;
    assert!(matches!(result, Err(UserAddErr::InviteNotFound)));

    let invite = server.invite_add("prime@heyadora.com").await.unwrap();
    // server.state.db.invi;

    // let result = server
    //     .post::<Result<UserAddRes, UserAddErr>>(
    //         USER_ADD_ENDPOINT,
    //         "username=hey&password=wtf&invite_token=invalid",
    //     )
    //     .await;
}

// pub async fn parse_multipart(mut multipart: Multipart) -> anyhow::Result<UserAddReq> {
//     let field_count = 3_usize;
//     let mut field_username = None::<String>;
//     let mut field_password = None::<String>;
//     let mut field_invite_token = None::<String>;
//     let mut index = 0;

//     while let Ok(Some(field)) = multipart.next_field().await {
//         if index >= field_count {
//             break;
//         }
//         let Some(name) = field.name() else {
//             break;
//         };
//         let name = match name {
//             "username" => {
//                 let bytes = field.bytes().await?;
//                 field_username = Some(String::from_utf8(bytes.to_vec())?);
//             }
//             "password" => {
//                 let bytes = field.bytes().await?;
//                 field_password = Some(String::from_utf8(bytes.to_vec())?);
//             }
//             "invite_token" => {
//                 let bytes = field.bytes().await?;
//                 field_invite_token = Some(String::from_utf8(bytes.to_vec())?);
//             }
//             _ => {
//                 break;
//             }
//         };
//         index += 1;
//     }

//     let username = field_username.ok_or_else(|| anyhow!("missing username"))?;
//     let password = field_password.ok_or_else(|| anyhow!("missing password"))?;
//     let invite_token = field_invite_token.ok_or_else(|| anyhow!("missing invite_token"))?;

//     Ok(UserAddReq {
//         username,
//         password,
//         invite_token,
//     })
// }
// let req = parse_multipart(multipart)
//     .await
//     .map_err(|err| UserAddErr::BadRequest(err.to_string()))?;
