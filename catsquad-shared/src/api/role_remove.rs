use std::fmt::Display;

use crate::RoleValue;

pub const LINK_API_ROLE_REMOVE: &str = "/api/role/{role_name}";

pub fn link_relative_role_remove(role_name: impl Display) -> String {
    format!("/api/role/{role_name}")
}

pub fn role_extract_permission_role_remove(roles: &[String]) -> bool {
    roles
        .iter()
        .any(|v| v == RoleValue::PermissionRoleRemove.as_str())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleRemovePrams {
    pub role_name: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum RoleRemoveErr {
    #[error("bad request {0}")]
    BadRequest(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
