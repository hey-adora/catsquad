use crate::{Uuid, serde_from_uuid, serde_to_uuid};

pub const LINK_API_ROLE_ADD: &str = "/api/role/{role_name}";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleRes {
    pub name: String,
    pub values: Vec<String>,
    pub use_username: String,
    pub created_at: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleAddPrams {
    pub role_name: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleAddReq {
    pub role_values: Vec<String>,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum RoleAddErr {
    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
