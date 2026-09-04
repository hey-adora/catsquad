pub const LINK_API_EMAIL_CHANGE_ADD: &str = "/api/email_change_update_current_add";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmailChangeRes {
    pub id: i64,
    pub current: EmailChangeToken,
    pub new: Option<EmailChangeToken>,
    pub completed: bool,
    pub expires: u64,
    pub modified_at: u64,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmailChangeToken {
    pub email: String,
    pub token_used: bool,
}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct EmailChangeAddReq {}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum EmailChangeAddErr {
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
