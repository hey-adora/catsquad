use crate::{Order, TimeRange, ToForm, serde_from_option_u128, serde_to_option_u128};
use catsquad_log::prelude::*;

pub const LINK_API_POST_SEARCH: &str = "/api/posts";

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
    pub tags: Option<String>,
    pub username: Option<String>,
    // #[serde(
    //     serialize_with = "serde_from_option_u128",
    //     deserialize_with = "serde_to_option_u128"
    // )]
    pub time: Option<String>,
    pub range: Option<TimeRange>,
    pub order: Option<Order>,
    pub limit: Option<usize>,
}

impl Default for PostSearchParams {
    fn default() -> Self {
        Self {
            time: Some(0.to_string()),
            range: Some(TimeRange::MoreOrEqual),
            order: Some(Order::ThreeTwoOne),
            limit: Some(50),
            tags: Some(String::new()),
            username: Some(String::new()),
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
