// use catsquad_log::prelude::*;

// pub const LINK_API_POST_UPDATE_FILE_ADD: &str = "/api/post_update_file_add";
pub const LINK_API_POST_UPDATE_FILE_ADD: &str = "/api/post/{post_key}";

pub fn link_relative_post_update_file_add(post_key: impl AsRef<str>) -> String {
    format!("/api/post/{}", post_key.as_ref())
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostFileGetByHashErr {
    #[error("post not found")]
    PostNotFound,

    #[error("post file not found")]
    FileNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServerErr,
}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct PostRes {
//     pub key: String,
//     pub user: UserRes,
//     pub show: bool,
//     pub title: String,
//     pub description: String,
//     pub tags: String,
//     pub favorites: u64,
//     pub file: Vec<PostFile>,
//     pub modified_at: u128,
//     pub created_at: u128,
// }

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct PostFile {
//     pub extension: String,
//     pub hash: String,
//     pub proccesed: bool,
//     pub size_bytes: usize,
//     pub width: u32,
//     pub height: u32,
// }

#[derive(
    Default, thiserror::Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub enum PostUpdateFileAddErr {
    #[error("file already exists")]
    Duplicate,

    #[error("no files found in your request data")]
    NotFilesFound,

    #[error("post id param not found")]
    ParamNotFoundPostId,

    #[error("ffmpeg err {0}")]
    ReadingResolutionErr(String),

    #[error("invalid resolution {width}x{height}")]
    InvalidResolution { width: u32, height: u32 },

    #[error("io error {0}")]
    IoErr(String),

    #[error("stream error {0}")]
    StreamErr(String),

    #[error("file {file_name} is too big, max file size {max}, stopped upload at: {got}")]
    FileTooBig {
        file_name: String,
        max: u64,
        got: u64,
    },

    #[error("post not found")]
    PostNotFound,

    #[error("file \"{0}\" must have extension in their name, such as .png")]
    FileHasNoExtension(String),

    #[error("file extension {0} is not supported")]
    UnsupportedExtension(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateFileAddReq {}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateFileAddParams {
    pub post_key: String,
}
pub const POST_UPDATE_FILE_ADD_PARAMS_FIELD_POST_KEY: &'static str = "post_key";

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
