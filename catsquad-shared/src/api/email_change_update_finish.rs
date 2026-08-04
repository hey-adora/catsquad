pub const LINK_API_EMAIL_CHANGE_UPDATE_FINISH: &str = "/api/email_change_update_finish";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailChangeUpdateFinishReq {
    pub email_change_key: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum EmailChangeUpdateFinishErr {
    #[error("not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("new email not set")]
    NewEmailNotConfirmed,

    #[error("email already taken")]
    EmailIsTaken,

    #[default]
    #[error("internal server err")]
    InternalServer,
}
