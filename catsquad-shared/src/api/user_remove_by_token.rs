use std::fmt::Display;

pub const LINK_API_USER_REMOVE_BY_TOKEN: &str = "/api/remove";

pub fn link_relative_user_remove_by_token() -> &'static str {
    LINK_API_USER_REMOVE_BY_TOKEN
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserRemoveByTokenReq {
    #[serde(
        serialize_with = "crate::serde_from_uuid",
        deserialize_with = "crate::serde_to_uuid"
    )]
    pub token: crate::Uuid,
    // pub token: String,
}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct UserRemoveByTokenParams {
//     pub user_username: String,
// }

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserRemoveByTokenErr {
    #[error("expired")]
    Expired,

    #[error("already used")]
    AlreadyUsed,

    #[error("password key not found")]
    TokenNotFound,

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
