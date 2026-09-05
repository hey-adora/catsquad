pub const LINK_API_PASSWORD_CHANGE_UPDATE_CONFIRM: &str = "/api/password_change_update_confirm";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PasswordChangeUpdateConfirmRes {
    // pub email: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PasswordChangeUpdateConfirmReq {
    #[serde(
        serialize_with = "crate::serde_from_uuid",
        deserialize_with = "crate::serde_to_uuid"
    )]
    pub token: crate::Uuid,
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
    TokenNotFound,

    #[error("new password is invalid")]
    NewPasswordInvalid(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
