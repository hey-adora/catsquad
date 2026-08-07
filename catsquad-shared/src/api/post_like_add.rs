use catsquad_log::prelude::*;

pub const LINK_API_POST_LIKE_ADD: &str = "/api/post_like_add";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostLikeRes {
    pub key: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostLikeAddReq {
    pub post_key: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostLikeAddErr {
    #[error("cant like your own post")]
    CantLikeYourself,

    #[error("post not found")]
    PostNotFound,

    #[error("post is already liked")]
    AlreadyLiked,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
