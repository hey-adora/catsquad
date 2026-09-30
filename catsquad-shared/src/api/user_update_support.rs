pub const LINK_API_USER_UPDATE_SUPPORT: &str = "/api/user_update_support";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateSupportReq {
    pub new_support: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserUpdateSupportErr {
    #[error("support is invalid {0}")]
    Invalid(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
