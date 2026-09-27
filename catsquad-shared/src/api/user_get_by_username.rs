use std::fmt::Display;

pub const LINK_API_USER_GET_BY_USERNAME: &str = "/api/user/{username}";

pub fn link_relative_user_get_by_username(username: impl Display) -> String {
    format!("/api/user/{}", username)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserGetByUsernameParams {
    pub username: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserGetByUsernameErr {
    #[error("found found")]
    NotFound,

    #[default]
    #[error("internal server err")]
    InternalServer,
}
