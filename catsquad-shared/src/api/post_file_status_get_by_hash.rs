pub const LINK_API_POST_FILE_STATUS_GET_BY_HASH: &str = "/api/file/{file_hash}/status";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostFileGetStatusByHashRes {
    pub is_proccesed: bool,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct PostFileGetStatusByHashParams {
    // pub post_id: i64,
    pub file_hash: i64,
}

pub fn link_relative_post_get_file_status_by_hash(file_hash: i64) -> String {
    format!("/api/file/{}/status", file_hash)
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostFileStatusGetByHashErr {
    #[error("post file not found")]
    FileNotFound,

    // #[error("unauthorized {0}")]
    // Unauthorized(String),
    #[default]
    #[error("internal server err")]
    InternalServerErr,
    // #[error("post not found")]
    // PostNotFound,

    // // #[error("bad request {0}")]
    // // BadRequest(String),
    // #[error("unauthorized {0}")]
    // Unauthorized(String),

    // #[default]
    // #[error("internal server err")]
    // InternalServerErr,
}
