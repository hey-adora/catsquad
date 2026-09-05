use crate::{Uuid, u128_to_str, uuid_to_str};

pub const LINK_API_INVITE_GET_BY_KEY: &str = "/api/invite/{invite_key}";

pub fn link_relative_invite_get_by_key(invite_token: Uuid) -> String {
    let uuid_str = uuid_to_str(invite_token);
    format!("/api/invite/{}", uuid_str)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteGetByKeyRes {
    pub email: String,
    pub expires: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteGetByKeyParams {
    pub invite_key: String,
}
pub const INVITE_GET_BY_KEY_REQ_FIELD_INVITE_KEY: &'static str = "invite_key";

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum InviteGetByKeyErr {
    #[error("invite not found")]
    InviteNotFound,

    #[error("invite already used")]
    InviteAlreadyUsed,

    #[error("invite expired")]
    InviteExpired,

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServerErr,
}
