use crate::RoleValue;
use std::fmt::Display;

// pub const LINK_API_ROLE_UPDATE_VALUE: &str = "/api/role/{role_id}";

pub fn link_relative_role_update_value(role_id: i64) -> String {
    format!("/api/role/{role_id}")
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleUpdateValueParams {
    pub role_name: i64, // role_id
}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct RoleAddPrams {
//     pub role_name: String,
// }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleUpdateValueReq {
    #[serde(
        serialize_with = "crate::serde_from_role_value",
        deserialize_with = "crate::serde_to_role_value"
    )]
    pub role_values: Vec<RoleValue>,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum RoleUpdateValueErr {
    #[error("duplicate role values")]
    DuplicateValue,

    #[error("role not found")]
    NotFound,

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
