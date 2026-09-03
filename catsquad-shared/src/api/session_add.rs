use crate::{Uuid, serde_from_uuid, serde_to_uuid};

pub const LINK_API_SESSION_ADD: &str = "/api/login";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionRes {
    // #[serde(serialize_with = "serde_from_uuid", deserialize_with = "serde_to_uuid")]
    // pub token: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionAddReq {
    pub email: String,
    pub password: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum SessionAddErr {
    #[error("email or password is wrong")]
    InvalidCredentials,

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
