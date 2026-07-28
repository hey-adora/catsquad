pub const LINK_API_EMAIL_CHANGE_UPDATE_CANCEL: &str = "/api/email_change_update_cancel";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailChangeUpdateCancelReq {
    pub email_change_key: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum EmailChangeUpdateCancelErr {
    #[error("not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("internal server err")]
    InternalServer,
}
