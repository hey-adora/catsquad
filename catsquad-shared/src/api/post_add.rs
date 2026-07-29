use std::fmt::Display;

use crate::{
    MAX_POST_DESCRIPTION_LENGTH, MAX_POST_TAGS_LENGTH, MAX_POST_TITLE_LENGTH, RedactedUserRes,
};
use catsquad_log::prelude::*;

pub const LINK_API_POST_ADD: &str = "/api/post";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostRes {
    pub key: String,
    pub user: RedactedUserRes,
    pub state: PostState,
    pub title: String,
    pub description: String,
    pub tags: String,
    pub favorites: u64,
    pub file: Vec<PostFile>,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PostState {
    Draft,
    Active,
    Hidden,
}

// never change this, requres database migration
pub const POST_STATE_DRAFT: &'static str = "draft";
pub const POST_STATE_ACTIVE: &'static str = "active";
pub const POST_STATE_HIDDEN: &'static str = "hidden";

impl From<String> for PostState {
    fn from(value: String) -> Self {
        let value = value.as_str();
        From::<&str>::from(value)
    }
}

impl From<&str> for PostState {
    fn from(value: &str) -> Self {
        match value {
            POST_STATE_DRAFT => Self::Draft,
            POST_STATE_ACTIVE => Self::Active,
            POST_STATE_HIDDEN => Self::Hidden,
            _ => unreachable!("database has invalid state"),
        }
    }
}

impl Display for PostState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PostState::Draft => POST_STATE_DRAFT,
            PostState::Active => POST_STATE_ACTIVE,
            PostState::Hidden => POST_STATE_HIDDEN,
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostFile {
    pub extension: String,
    pub hash: String,
    pub proccesed: bool,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostAddReq {
    pub title: String,
    pub description: String,
    pub tags: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostAddErr {
    #[error("failed to create dir {0}")]
    ServerDirCreationFailed(String),

    #[error("file system err {0}")]
    ServerFSErr(String),

    #[error("invalid tags {0}")]
    InvalidTags(String),

    #[error("invalid title {0}")]
    InvalidTitle(String),

    #[error("invalid description {0}")]
    InvalidDescription(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
    // #[error("img proccesing error {0:#?}")]
    // ServerImgErr(Vec<ServerErrImgMeta>),
    // #[error("email is invalid")]
    // InvalidEmail(String),

    // #[error("bad request {0}")]
    // BadRequest(String),
}

pub fn validate_post_tags<S: AsRef<str>>(tags: S) -> Result<(), String> {
    let mut errors = String::new();
    let input = tags.as_ref();

    if MAX_POST_TAGS_LENGTH < input.len() {
        errors += "tags max length is 2000 characters\n";
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}

#[test]
fn test_validate_post_tags() {
    use rand::distr::SampleString;

    assert!(validate_post_tags("").is_ok());
    assert!(validate_post_tags("www").is_ok());

    let tmp = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), MAX_POST_TAGS_LENGTH);
    assert!(validate_post_tags(tmp).is_ok());

    let tmp = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), MAX_POST_TAGS_LENGTH + 1);
    assert!(validate_post_tags(tmp).is_err());
}

pub fn validate_post_title<S: AsRef<str>>(title: S) -> Result<(), String> {
    let mut errors = String::new();
    let input = title.as_ref();

    if MAX_POST_TITLE_LENGTH < input.len() {
        errors += "max title length is 120 characters\n";
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}

#[test]
fn test_validate_post_title() {
    use rand::distr::SampleString;

    assert!(validate_post_title("").is_ok());
    assert!(validate_post_title("www").is_ok());

    let tmp = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), MAX_POST_TITLE_LENGTH);
    assert!(validate_post_title(tmp).is_ok());

    let tmp = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), MAX_POST_TITLE_LENGTH + 1);
    assert!(validate_post_title(tmp).is_err());
}

pub fn validate_post_description<S: AsRef<str>>(description: S) -> Result<(), String> {
    let mut errors = String::new();
    let input = description.as_ref();

    if input.len() > MAX_POST_DESCRIPTION_LENGTH {
        errors += "max description length is 2000 characters\n";
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}

#[test]
fn test_validate_post_description() {
    use rand::distr::SampleString;

    assert!(validate_post_description("").is_ok());
    assert!(validate_post_description("www").is_ok());

    let tmp =
        rand::distr::Alphanumeric.sample_string(&mut rand::rng(), MAX_POST_DESCRIPTION_LENGTH);
    assert!(validate_post_description(tmp).is_ok());

    let tmp =
        rand::distr::Alphanumeric.sample_string(&mut rand::rng(), MAX_POST_DESCRIPTION_LENGTH + 1);
    assert!(validate_post_description(tmp).is_err());
}

// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
// pub struct ServerErrImgMeta {
//     pub path: String,
//     pub err: ServerErrImg,
// }

// #[derive(thiserror::Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
// pub enum ServerErrImg {
//     #[error("failed to read img metadata {0}")]
//     ServerImgMetadataReadFail(String),

//     #[error("unsupported format {0}")]
//     ServerImgUnsupportedFormat(String),

//     #[error("img decode failed {0}")]
//     ServerImgDecodeFailed(String),

//     #[error("failed to create img webp encoder {0}")]
//     ServerImgWebPEncoderCreationFailed(String),

//     #[error("failed to encode img as webp {0}")]
//     ServerImgWebPEncodingFailed(String),
// }
