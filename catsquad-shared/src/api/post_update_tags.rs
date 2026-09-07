pub const LINK_API_POST_UPDATE_TAGS: &str = "/api/post_update_tags";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostUpdateTagsReq {
    pub post_id: i64,
    pub new_tags: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostUpdateTagsErr {
    #[error("post not found")]
    PostNotFound,

    #[error("tags is invalid {0}")]
    InvalidTags(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}

pub fn proccess_tags(tags: impl Into<String>) -> String {
    let tags = tags.into();
    let tags_len = tags.len();
    let tags = tags.to_lowercase();
    let tags_iter = tags.split_ascii_whitespace();
    let mut tags = String::with_capacity(tags_len);
    tags.push(' ');
    for tag in tags_iter {
        tags.push_str(tag);
        tags.push(' ');
    }

    // no tags = clear spaces
    if tags.len() == 1 {
        tags.clear();
    }

    tags
}
