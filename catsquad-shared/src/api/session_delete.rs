pub const LINK_API_SESSION_DELETE: &str = "/api/logout";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionDeleteRes {}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct SessionDeleteReq {}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum SessionDeleteErr {
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}
