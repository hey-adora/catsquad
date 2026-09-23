pub const LINK_API_POST_UPDATE_IMAGE_ORD: &str = "/api/post_update_file_ord";

pub fn link_relative_post_update_image_ord() -> &'static str {
    LINK_API_POST_UPDATE_IMAGE_ORD
}

#[derive(
    Default, thiserror::Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub enum PostUpdateImageOrdErr {
    #[error("post not found")]
    PostNotFound,

    #[error("invalid selected index")]
    InvalidIndex,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateImageOrdReq {
    pub post_id: i64,
    pub image_index_a: usize,
    pub image_index_b: usize,
}

pub fn arr_remove_and_insert<T>(arr: &mut Vec<T>, pos_a: usize, pos_b: usize) {
    let hash = arr.remove(pos_a);
    arr.insert(pos_b, hash);
}
