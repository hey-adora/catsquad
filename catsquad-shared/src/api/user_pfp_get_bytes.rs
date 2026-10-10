use std::fmt::Display;

pub const LINK_API_USER_PFP_GET_BYTES: &str = "/api/user/{username}/pfp";

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct UserPfpGetBytesParams {
    pub username: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserPfpGetBytesErr {
    #[error("pfp not proccesed yet")]
    StillProccesing,

    #[error("pfp not found")]
    PfpNotFound,

    #[error("user not found")]
    UserNotFound,

    #[default]
    #[error("internal server err")]
    InternalServerErr,
}

pub fn link_relative_user_pfp_get_bytes(username: impl Display) -> String {
    format!("/api/user/{}/pfp", username)
}
