pub const LINK_API_TOKEN_USER_REMOVE_ADD: &str = "/api/token_user_remove_add";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TokenUserRemoveRes {
    pub expires: u64,
}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct TokenUserRemoveAddReq {
//     pub email: String,
// }

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum TokenUserRemoveAddErr {
    // #[error("email is invalid")]
    // InvalidEmail(String),
    // #[error("bad request {0}")]
    // BadRequest(String),
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
