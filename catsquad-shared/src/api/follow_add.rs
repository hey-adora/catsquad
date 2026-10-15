use std::fmt::Display;

use catsquad_log::prelude::*;
pub const LINK_API_FOLLOW_ADD: &str = "/api/follow/{user_username}";

pub fn link_relative_follow_add(username: impl Display) -> String {
    format!("/api/follow/{username}")
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FollowRes {
    pub id: i64,
    pub follower_user_username: String,
    pub following_user_username: String,
    pub modified_at: u64,
    pub created_at: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FollowAddParams {
    pub user_username: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum FollowAddErr {
    #[error("cant follow yourself")]
    CantFollowYourself,

    #[error("user is already followed")]
    UserAlreadyFollowed,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
