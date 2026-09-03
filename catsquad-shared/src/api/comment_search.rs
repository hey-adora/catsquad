use crate::{Order, TimeRange, ToForm, serde_from_u128, serde_to_u128};
use catsquad_log::prelude::*;

pub const LINK_API_COMMENT_SEARCH: &str = "/api/comment_search";

pub fn link_relative_comment_search(options: CommentSearchParams) -> String {
    format!("{LINK_API_COMMENT_SEARCH}?{}", options.to_form().unwrap())
}
// url::Url::parse(LINK_API_COMMENT_SEARCH).unwrap();
// .inspect_err(|err| error!("invalid comment search params {options:?} {err}"))
// .unwrap_or_else(|_| CommentSearchParams::default().)

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CommentSearchParams {
    pub post_id: i64,
    pub comment_id: i64,
    pub time: u64,
    pub limit: usize,
    pub range: TimeRange,
    pub order: Order,
    pub flatten: bool,
}

// #[serde(serialize_with = "serde_from_u128", deserialize_with = "serde_to_u128")]
impl Default for CommentSearchParams {
    fn default() -> Self {
        Self {
            post_id: 0,
            comment_id: 0,
            time: 0,
            limit: 50,
            range: TimeRange::MoreOrEqual,
            order: Order::ThreeTwoOne,
            flatten: false,
        }
    }
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum CommentSearchErr {
    #[default]
    #[error("internal server err")]
    InternalServer,
}
