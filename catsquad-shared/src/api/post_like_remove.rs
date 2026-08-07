use catsquad_log::prelude::*;

pub const LINK_API_POST_LIKE_REMOVE: &str = "/api/post_like_remove";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostLikeRemoveReq {
    pub post_key: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostLikeRemoveErr {
    #[error("like not found")]
    LikeNotFound,

    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
