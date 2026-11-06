pub const LINK_API_CONFIRM_TOKEN_PASSWORD_CHANGE_ADD: &str =
    "/api/confirm_token_password_change_add";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ConfirmTokenPasswordChangeRes {
    pub expires: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ConfirmTokenPasswordChangeAddReq {
    pub email: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum ConfirmTokenPasswordChangeAddErr {
    #[error("email is invalid")]
    InvalidEmail(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
