pub const LINK_API_POST_REMOVE: &str = "/api/post/{post_key}";

pub fn link_relative_post_remove(post_key: impl AsRef<str>) -> String {
    format!("/api/post/{}", post_key.as_ref())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostRemoveParams {
    pub post_key: String,
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
