pub const LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_ADD: &str = "/api/email_change_update_current_add";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmailChangeRes {
    pub key: String,
    pub current: EmailChangeToken,
    pub new: Option<EmailChangeToken>,
    pub completed: bool,
    pub expires: u128,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmailChangeToken {
    pub email: String,
    pub token_used: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailChangeUpdateCurrentAddReq {}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum EmailChangeUpdateCurrentAddErr {
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("internal server err")]
    InternalServer,
}
