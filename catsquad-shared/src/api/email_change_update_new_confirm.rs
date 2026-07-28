pub const LINK_API_EMAIL_CHANGE_UPDATE_NEW_CONFIRM: &str = "/api/email_change_update_new_confirm";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailChangeUpdateNewConfirmReq {
    pub email_change_key: String,
    pub token: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum EmailChangeUpdateNewConfirmErr {
    #[error("not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("new email not set")]
    NewEmailNotSet,

    #[error("invalid token")]
    InvalidToken,

    #[error("internal server err")]
    InternalServer,
}
