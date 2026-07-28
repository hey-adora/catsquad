pub const LINK_API_SESSION_ADD: &str = "/api/login";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionRes {
    pub key: String,
    pub username: String,
    pub email: String,
    pub created_at: u128,
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
