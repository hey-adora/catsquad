pub const LINK_API_EMAIL_CHANGE_RESEND: &str = "/api/email_change_resend";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailChangeResendReq {
    pub email_change_key: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum EmailChangeResendErr {
    #[error("not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("email change is not in email confirmation state")]
    NothingToResend,

    #[default]
    #[error("internal server err")]
    InternalServer,
}
