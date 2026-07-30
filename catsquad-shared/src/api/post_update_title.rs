pub const LINK_API_POST_UPDATE_TITLE: &str = "/api/post_update_title";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateTitleReq {
    pub post_key: String,
    pub new_title: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostUpdateTitleErr {
    #[error("post not found")]
    PostNotFound,

    #[error("title is invalid {0}")]
    InvalidTitle(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
