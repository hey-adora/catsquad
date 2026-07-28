pub const LINK_API_INVITE_ADD: &str = "/api/invite";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteRes {
    pub expires: u128,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteAddReq {
    pub email: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum InviteAddErr {
    #[error("email is invalid")]
    InvalidEmail(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
