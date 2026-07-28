pub const LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_CONFIRM: &str =
    "/api/email_change_update_current_confirm";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailChangeUpdateCurrentConfirmReq {
    pub email_change_key: String,
    pub token: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum EmailChangeUpdateCurrentConfirmErr {
    #[error("not found")]
    NotFound,

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("invalid token")]
    InvalidToken,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("internal server err")]
    InternalServer,
}
