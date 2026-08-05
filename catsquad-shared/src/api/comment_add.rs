use crate::{MAX_POST_COMMENT_LENGTH, RedactedUserRes};
use catsquad_log::prelude::*;

pub const LINK_API_COMMENT_ADD: &str = "/api/comment_add";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CommentRes {
    pub key: String,
    pub user: RedactedUserRes,
    pub post_key: String,
    pub parent_key: Vec<String>,
    pub text: String,
    pub replies_count: usize,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CommentAddReq {
    pub post_key: String,
    pub comment_key: Option<String>,
    pub text: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum CommentAddErr {
    #[error("post \"{0}\" was not found")]
    PostNotFound(String),

    #[error("reply_comment \"{0}\" was not found")]
    ReplyCommentNotFound(String),

    #[error("invalid text {0}")]
    InvalidText(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}

pub fn validate_comment_text<S: AsRef<str>>(comment: S) -> Result<(), String> {
    let mut errors = String::new();
    let input = comment.as_ref();

    if input.is_empty() {
        errors += "comment cant be empty\n";
    }

    if input.len() > MAX_POST_COMMENT_LENGTH {
        errors += "max comment length is 2000 chars\n";
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}
