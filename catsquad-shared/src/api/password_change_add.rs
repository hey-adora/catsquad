pub const LINK_API_PASSWORD_CHANGE_ADD: &str = "/api/password_change_add";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PasswordChangeRes {
    pub expires: u128,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PasswordChangeReq {
    pub email: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum PasswordChangeAddErr {
    #[error("email is invalid")]
    InvalidEmail(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("internal server err")]
    InternalServer,
}
