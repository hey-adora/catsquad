pub const LINK_API_POST_UPDATE_FILE_REMOVE: &str = "/api/post_update_file_remove";

pub fn link_relative_post_update_file_remove() -> &'static str {
    LINK_API_POST_UPDATE_FILE_REMOVE
}

#[derive(
    Default, thiserror::Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub enum PostUpdateFileRemoveErr {
    #[error("no files found in your request data")]
    FileNotFound,

    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateFileRemoveReq {
    pub post_key: String,
    pub hash: String,
}
