use std::fmt::Display;

pub const LINK_API_USER_GET_BY_USERNAME: &str = "/api/user/{username}";

pub fn link_relative_user_get_by_username(username: impl Display) -> String {
    format!("/api/user/{}", username)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserGetRes {
    // pub key: i64,
    pub username: String,
    pub pfp_hash: i64,
    pub pfp_proccesed: bool,
    pub likes: u32,
    pub followers: u32,
    pub followed: bool,
    pub support: String,
    pub aboutme: String,
    pub created_at: u64,
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
