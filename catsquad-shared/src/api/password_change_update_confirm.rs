pub const LINK_API_PASSWORD_CHANGE_UPDATE_CONFIRM: &str = "/api/password_change_update_confirm";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PasswordChangeUpdateConfirmRes {
    //
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PasswordChangeUpdateConfirmReq {
    pub password_change_key: String,
    pub new_password: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PasswordChangeUpdateConfirmErr {
    #[error("expired")]
    Expired,

    #[error("already used")]
    AlreadyUsed,

    #[error("password key not found")]
    PasswordKeyNotFound,

    #[error("new password is invalid")]
    NewPasswordInvalid(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
