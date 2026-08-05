pub const LINK_API_COMMENT_UPDATE_TEXT: &str = "/api/comment_update_text";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CommentUpdateTextReq {
    pub comment_key: String,
    pub text: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum CommentUpdateTextErr {
    #[error("post was not found")]
    PostNotFound,

    #[error("invalid text {0}")]
    InvalidText(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
