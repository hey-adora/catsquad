#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserAddReq {
    pub username: String,
    pub password: String,
    pub invite_token: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserAddRes {
    pub key: String,
    pub username: String,
    pub email: String,
    pub created_at: u128,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum UserAddErr {
    #[error("invalid input")]
    InvalidInput {
        username: Option<String>,
        password: Option<String>,
    },

    #[error("email is taken")]
    EmailIsTaken,

    #[error("username is taken")]
    UsernameIsTaken,

    #[error("invite not found")]
    InviteNotFound,

    #[error("invite already used")]
    InviteAlreadyUsed,

    #[error("invite expired")]
    InviteExpired,

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("internal server err")]
    InternalServer,
}
