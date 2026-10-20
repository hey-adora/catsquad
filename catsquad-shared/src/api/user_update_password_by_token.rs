pub const LINK_API_USER_UPDATE_PASSWORD_BY_TOKEN: &str = "/api/user_update_password_by_token";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdatePasswordByTokenRes {
    pub email: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdatePasswordByTokenReq {
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
pub enum UserUpdatePasswordByTokenErr {
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
