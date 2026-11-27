pub const LINK_API_ROLE_GET_ALL: &str = "/api/role";

pub fn link_relative_role_get_all() -> &'static str {
    "/api/role"
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum RoleGetAllErr {
    #[default]
    #[error("internal server err")]
    InternalServer,
}
