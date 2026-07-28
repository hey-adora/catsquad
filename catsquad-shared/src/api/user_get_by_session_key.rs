pub const LINK_API_SESSION_GET_BY_SESSION_KEY: &str = "/api/profile";

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserGetBySessionKeyErr {
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
