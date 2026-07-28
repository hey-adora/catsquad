pub const LINK_API_EMAIL_CHANGE_UPDATE_NEW_ADD: &str = "/api/email_change_update_new_add";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailChangeUpdateNewAddReq {
    pub email_change_key: String,
    pub new_email: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum EmailChangeNewAddErr {
    #[error("not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("current email not confirmed")]
    NotConfirmed,

    #[error("email {0} already taken")]
    EmailIsTaken(String),

    #[error("internal server err")]
    InternalServer,
}
