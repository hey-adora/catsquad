pub const LINK_API_POST_LIKE_GET_BY_POST: &str = "/api/post/{post_key}/like";

pub fn link_relative_post_like_get_by_post(post_key: impl AsRef<str>) -> String {
    format!("/api/post/{}/like", post_key.as_ref())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostLikeGetByPostParams {
    pub post_key: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostLikeGetByPostErr {
    #[default]
    #[error("internal server err")]
    InternalServer,
}
