pub const LINK_API_TOKEN_PASSWORD_CHANGE_ADD: &str = "/api/token_password_change_add";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TokenPasswordChangeRes {
    pub expires: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TokenPasswordChangeAddReq {
    pub email: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum TokenPasswordChangeAddErr {
    #[error("email is invalid")]
    InvalidEmail(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
