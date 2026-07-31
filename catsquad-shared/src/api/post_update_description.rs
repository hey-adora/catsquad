pub const LINK_API_POST_UPDATE_DESCRIPTION: &str = "/api/post_update_description";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateDescriptionReq {
    pub post_key: String,
    pub new_description: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostUpdateDescriptionErr {
    #[error("post not found")]
    PostNotFound,

    #[error("description is invalid {0}")]
    InvalidDescription(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
