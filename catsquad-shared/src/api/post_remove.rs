pub const LINK_API_POST_REMOVE: &str = "/api/post/{post_id}";

pub fn link_relative_post_remove(post_id: i64) -> String {
    format!("/api/post/{}", post_id)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostRemoveParams {
    pub post_id: i64,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostRemoveErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
