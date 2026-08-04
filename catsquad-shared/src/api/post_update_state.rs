use crate::PostState;

pub const LINK_API_POST_UPDATE_STATE: &str = "/api/post_update_state";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateStateReq {
    pub post_key: String,
    pub new_state: PostState,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostUpdateStateErr {
    #[error("same state")]
    SameState,

    #[error("cant set draft")]
    CantSetDraft,

    #[error("not active")]
    PostNotActive,

    #[error("post not found")]
    PostNotFound,

    #[error("user not found")]
    UserNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
