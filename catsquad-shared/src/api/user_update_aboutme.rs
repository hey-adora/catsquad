pub const LINK_API_USER_UPDATE_ABOUTME: &str = "/api/user_update_aboutme";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateAboutmeReq {
    pub new_aboutme: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserUpdateAboutmeErr {
    #[error("aboutme is invalid {0}")]
    Invalid(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
