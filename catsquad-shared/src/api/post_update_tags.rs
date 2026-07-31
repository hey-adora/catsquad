pub const LINK_API_POST_UPDATE_TAGS: &str = "/api/post_update_tags";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateTagsReq {
    pub post_key: String,
    pub new_tags: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostUpdateTagsErr {
    #[error("post not found")]
    PostNotFound,

    #[error("tags is invalid {0}")]
    InvalidTags(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
