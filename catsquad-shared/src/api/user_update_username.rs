pub const LINK_API_USER_UPDATE_USERNAME: &str = "/api/user_update_username";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateUsernameRes {
    pub username: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateUsernameReq {
    // pub email: String,
    pub password: String,
    pub new_username: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum UserUpdateUsernameErr {
    #[error("username alreay used")]
    UsernameAlreadyUsed,

    #[error("username is invalid {0}")]
    InvalidUsername(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("internal server err")]
    InternalServer,
}
