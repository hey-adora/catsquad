pub const LINK_API_EMAIL_CHANGE_GET_BY_ID: &str = "/api/email_change/{email_change_id}";

pub fn link_relative_email_change_get_by_id(email_change_id: i64) -> String {
    format!("/api/email_change/{}", email_change_id)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailChangeGetByIdParams {
    pub email_change_id: i64,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum EmailChangeGetByIdErr {
    #[error("not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[default]
    #[error("internal server err")]
    InternalServer,
}
