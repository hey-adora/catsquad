use std::fmt::Display;

pub const LINK_API_FOLLOW_REMOVE: &str = "/api/follow/{user_username}";

pub fn link_relative_follow_remove(username: impl Display) -> String {
    format!("/api/follow/{username}")
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FollowRemoveParams {
    pub user_username: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum FollowRemoveErr {
    #[error("not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
