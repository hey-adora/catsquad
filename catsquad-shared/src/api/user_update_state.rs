use std::fmt::Display;

use crate::UserState;

pub const LINK_API_USER_UPDATE_STATE: &str = "/api/user/{username}/state/{state}";

pub fn link_relative_api_user_update_state(
    username: impl Display,
    user_state: UserState,
) -> String {
    format!("/api/user/{}/state/{}", username, user_state.as_str())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateStateRes {}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateStateParams {
    pub username: String,
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
    #[error("user not found")]
    UserNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
