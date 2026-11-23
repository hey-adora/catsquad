use crate::{ToForm, UserSearchParams};
use catsquad_log::prelude::*;

pub const LINK_API_USER_PAGINATION: &str = "/api/users/pagination";

pub fn link_relative_user_pagination(params: UserSearchParams) -> String {
    format!(
        "{LINK_API_USER_PAGINATION}?{}",
        params
            .to_form()
            .inspect_err(|err| error!("{} {:#?}", err, params))
            .unwrap()
    )
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserPaginationRes {
    pub total_count: usize,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserPaginationErr {
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
