use crate::{Order, TimeRange, ToForm, serde_from_u128, serde_to_u128};
use catsquad_log::prelude::*;

pub const LINK_API_POST_SEARCH: &str = "/api/post_search";

pub fn link_relative_post_search(params: PostSearchParams) -> String {
    format!(
        "{LINK_API_POST_SEARCH}?{}",
        params
            .to_form()
            .inspect_err(|err| error!("{:#?}", params))
            .unwrap()
    )
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PostSearchParams {
    pub tags: String,
    pub username: String,
    #[serde(serialize_with = "serde_from_u128", deserialize_with = "serde_to_u128")]
    pub time: u128,
    pub range: TimeRange,
    pub order: Order,
    pub limit: usize,
}

impl Default for PostSearchParams {
    fn default() -> Self {
        Self {
            time: 0,
            range: TimeRange::MoreOrEqual,
            order: Order::ThreeTwoOne,
            limit: 50,
            tags: String::new(),
            username: String::new(),
        }
    }
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostSearchErr {
    #[default]
    #[error("internal server err")]
    InternalServer,
}
