use crate::{PostFile, PostState};

pub const LINK_API_POST_GET_BY_ID: &str = "/api/post/{post_id}";

pub fn link_relative_post_get_by_key(post_id: i64) -> String {
    format!("/api/post/{}", post_id)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostGetRes {
    pub id: i64,
    pub user_username: String,
    pub state: PostState,
    pub title: String,
    pub description: String,
    pub tags: String,
    pub favorites: u32,
    pub file: Vec<PostFile>,
    pub modified_at: u64,
    pub created_at: u64,
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
