use std::fmt::Display;

pub const LINK_API_USER_REMOVE_BY_USERNAMES: &str = "/api/remove_by_usernames";

pub fn link_relative_user_remove_by_usernames() -> &'static str {
    LINK_API_USER_REMOVE_BY_USERNAMES
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserRemoveByUsernamesReq {
    #[serde(
        serialize_with = "crate::serde_from_string_vec",
        deserialize_with = "crate::serde_to_string_vec"
    )]
    pub usernames: Vec<String>,
    // pub token: String,
}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct UserRemoveByTokenParams {
//     pub user_username: String,
// }

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserRemoveByUsernamesErr {
    // #[error("expired")]
    // Expired,

    // #[error("already used")]
    // AlreadyUsed,

    // #[error("password key not found")]
    // TokenNotFound,

    // #[error("bad request {0}")]
    // BadRequest(String),
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
