use std::fmt::Display;

use crate::RoleValue;

// pub const LINK_API_ROLE_REMOVE: &str = "/api/role/{role_id}";

pub fn link_relative_role_remove(role_id: i64) -> String {
    format!("/api/role/{role_id}")
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleRemoveParams {
    pub role_name: i64, // role_id
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum RoleRemoveErr {
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
