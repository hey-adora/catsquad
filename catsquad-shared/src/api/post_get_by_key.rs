pub const LINK_API_POST_GET_BY_KEY: &str = "/api/post/{post_key}";

pub fn link_relative_post_get_by_key(post_id: i64) -> String {
    format!("/api/post/{}", post_id)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostGetByKeyParams {
    pub post_id: i64,
}
// pub const POST_GET_BY_KEY_REQ_FIELD_POST_KEY: &'static str = "post_key";

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostGetByKeyErr {
    #[error("post not found")]
    PostNotFound,

    // #[error("bad request {0}")]
    // BadRequest(String),
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServerErr,
}
