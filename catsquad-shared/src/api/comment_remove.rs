pub const LINK_API_COMMENT_REMOVE: &str = "/api/comment_remove";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CommentRemoveReq {
    pub comment_key: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum CommentRemoveErr {
    #[error("post was not found")]
    CommentNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
