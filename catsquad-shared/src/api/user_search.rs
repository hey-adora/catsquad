use crate::{Order, TimeRange, ToForm};
use catsquad_log::prelude::*;

pub const LINK_API_USER_SEARCH: &str = "/api/users";

pub fn link_relative_user_search(params: UserSearchParams) -> String {
    format!(
        "{LINK_API_USER_SEARCH}?{}",
        params
            .to_form()
            .inspect_err(|err| error!("{} {:#?}", err, params))
            .unwrap()
    )
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserSearchRes {
    pub username: String,
    pub pfp_hash: i64,
    pub pfp_proccesed: bool,
    pub likes: u32,
    pub following: u32,
    pub followers: u32,
    pub support: String,
    pub aboutme: String,
    pub created_at: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserSearchParams {
    pub time: Option<u64>,
    pub range: Option<TimeRange>,
    pub order: Option<Order>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl Default for UserSearchParams {
    fn default() -> Self {
        Self {
            time: Some(0),
            range: Some(TimeRange::MoreOrEqual),
            order: Some(Order::ThreeTwoOne),
            limit: Some(100),
            offset: Some(0),
        }
    }
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserSearchErr {
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
