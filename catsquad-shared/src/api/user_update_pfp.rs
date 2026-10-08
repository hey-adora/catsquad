use crate::FileImageAddErr;

pub const LINK_API_USER_UPDATE_PFP: &str = "/api/user_update_pfp";

pub fn link_relative_user_update_pfp() -> &'static str {
    LINK_API_USER_UPDATE_PFP
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserUpdatePfpErr {
    #[error("user not found")]
    IsSame,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[error(transparent)]
    Image(#[from] FileImageAddErr),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
