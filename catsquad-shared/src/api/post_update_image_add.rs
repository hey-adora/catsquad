// use catsquad_log::prelude::*;

use crate::{FileImage, FileImageAddErr};

pub const LINK_API_POST_UPDATE_FILE_ADD: &str = "/api/post/{post_id}/image";

pub fn link_relative_post_update_file_add(post_id: i64) -> String {
    format!("/api/post/{}/image", post_id)
}

#[derive(
    Default, thiserror::Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub enum PostUpdateImageAddErr {
    #[error("file already exists")]
    Duplicate,

    #[error("no files found in your request data")]
    NotImagesFound,

    #[error("post id param not found")]
    ParamNotFoundPostId,

    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error(transparent)]
    Image(#[from] FileImageAddErr),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct PostUpdateFileAddReq {}
pub type PostUpdateImageAddRes = FileImage;
// pub type PostUpdateImageAddRes = Vec<FileImage>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateImageAddParams {
    pub post_id: i64,
}
pub const POST_UPDATE_IMAGE_ADD_PARAMS_FIELD_POST_ID: &'static str = "post_id";

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct PostUpdateFileAddReq {
//     pub title: String,
//     pub description: String,
//     pub tags: String,
// }

// #[derive(
//     Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
// )]
// pub enum PostAddErr {
//     #[error("failed to create dir {0}")]
//     ServerDirCreationFailed(String),

//     #[error("file system err {0}")]
//     ServerFSErr(String),

//     #[error("invalid tags {0}")]
//     InvalidTags(String),

//     #[error("invalid title {0}")]
//     InvalidTitle(String),

//     #[error("invalid description {0}")]
//     InvalidDescription(String),

//     #[error("unauthorized {0}")]
//     Unauthorized(String),

//     #[default]
//     #[error("internal server err")]
//     InternalServer,
// }
