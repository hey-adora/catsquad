use crate::UserState;

pub const LINK_API_USER_UPDATE_STATE: &str = "/api/users/update/state";

pub fn link_relative_api_user_update_state() -> &'static str {
    LINK_API_USER_UPDATE_STATE
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserUpdateStateRes {
    pub username: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateStateReq {
    #[serde(
        serialize_with = "crate::serde_from_string_vec",
        deserialize_with = "crate::serde_to_string_vec"
    )]
    pub usernames: Vec<String>,
    #[serde(
        serialize_with = "crate::serde_from_user_state",
        deserialize_with = "crate::serde_to_user_state"
    )]
    pub state: UserState,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserUpdateStateErr {
    // #[error("user not found")]
    // UserNotFound,
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
