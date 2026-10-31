use catsquad_log::prelude::*;
use std::fmt::Display;

use anyhow::anyhow;

/// also used in /shared/src/api/role_remove.rs
pub const LINK_API_ROLE: &str = "/api/role/{role_name}";

pub const ROLE_VALUE_PERMISSION_ROLE_ADD: &'static str = "permission_role_add";
pub const ROLE_VALUE_PERMISSION_ROLE_REMOVE: &'static str = "permission_role_remove";
pub const ROLE_VALUE_PERMISSION_USER_SEARCH: &'static str = "permission_user_search";

pub fn link_relative_role_add(role_name: impl Display) -> String {
    format!("/api/role/{role_name}")
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleRes {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum RoleValue {
    PermissionRoleAdd,
    PermissionRoleRemove,
    PermissionUserSearch,
}

pub fn contains_roles<T>(expected_roles: &[RoleValue], roles_values: &[T]) -> bool
where
    T: TryInto<RoleValue> + Clone,
{
    expected_roles.iter().all(|expected_role| {
        roles_values.iter().any(|role_value| {
            let Ok(role_value): Result<RoleValue, T::Error> = role_value.clone().try_into() else {
                debug!("invalid value");
                return false;
            };
            trace!(
                "expected {expected_role} == {role_value } = {}",
                *expected_role == role_value
            );
            role_value == *expected_role
        })
    })
}

#[test]
fn test_contains_roles() {
    init_log();

    assert!(contains_roles(
        &[
            RoleValue::PermissionRoleAdd,
            RoleValue::PermissionRoleRemove,
            RoleValue::PermissionUserSearch,
        ],
        &[
            "permission_role_add",
            "permission_role_remove",
            "permission_user_search",
        ],
    ));
    return;

    assert!(contains_roles(
        &[RoleValue::PermissionRoleAdd],
        &[RoleValue::PermissionRoleAdd],
    ));

    assert!(!contains_roles(
        &[RoleValue::PermissionRoleAdd],
        &[RoleValue::PermissionRoleRemove],
    ));

    assert!(contains_roles(
        &[RoleValue::PermissionRoleAdd],
        &[
            RoleValue::PermissionRoleRemove,
            RoleValue::PermissionRoleAdd
        ],
    ));

    assert!(!contains_roles(
        &[
            RoleValue::PermissionRoleAdd,
            RoleValue::PermissionUserSearch
        ],
        &[
            RoleValue::PermissionRoleRemove,
            RoleValue::PermissionRoleAdd
        ],
    ));

    assert!(contains_roles(
        &[
            RoleValue::PermissionRoleAdd,
            RoleValue::PermissionUserSearch
        ],
        &[
            RoleValue::PermissionRoleRemove,
            RoleValue::PermissionRoleAdd,
            RoleValue::PermissionUserSearch
        ],
    ));
}

pub fn role_extract_permission_role_add(roles_values: &[String]) -> bool {
    roles_values
        .iter()
        .any(|v| v == RoleValue::PermissionRoleAdd.as_str())
}

pub fn role_extract_permission_role_remove(roles_values: &[String]) -> bool {
    roles_values
        .iter()
        .any(|v| v == RoleValue::PermissionRoleRemove.as_str())
}

pub fn role_extract_permission_user_search(roles_values: &[String]) -> bool {
    roles_values
        .iter()
        .any(|v| v == RoleValue::PermissionUserSearch.as_str())
}

impl RoleValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PermissionRoleAdd => ROLE_VALUE_PERMISSION_ROLE_ADD,
            Self::PermissionRoleRemove => ROLE_VALUE_PERMISSION_ROLE_REMOVE,
            Self::PermissionUserSearch => ROLE_VALUE_PERMISSION_USER_SEARCH,
        }
    }
}

impl From<RoleValue> for String {
    fn from(value: RoleValue) -> Self {
        value.to_string()
    }
}

impl TryFrom<&str> for RoleValue {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let result = match value {
            ROLE_VALUE_PERMISSION_ROLE_ADD => Self::PermissionRoleAdd,
            ROLE_VALUE_PERMISSION_ROLE_REMOVE => Self::PermissionRoleRemove,
            ROLE_VALUE_PERMISSION_USER_SEARCH => Self::PermissionUserSearch,
            _ => return Err(anyhow!("invalid value {value}")),
        };

        Ok(result)
    }
}

impl TryFrom<String> for RoleValue {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        TryFrom::try_from(value.as_str())
    }
}

impl Display for RoleValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self.as_str();
        write!(f, "{}", text)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleAddPrams {
    pub role_name: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoleAddReq {
    #[serde(
        serialize_with = "crate::serde_from_role_value",
        deserialize_with = "crate::serde_to_role_value"
    )]
    pub role_values: Vec<RoleValue>,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum RoleAddErr {
    #[error("duplicate role values")]
    DuplicateValue,

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
