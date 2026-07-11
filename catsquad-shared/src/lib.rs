mod api;

pub const LINK_API_USER_ADD: &str = "/api/register";
pub const LINK_API_INVITE_ADD: &str = "/api/invite";
pub const LINK_API_INVITE_GET_BY_KEY: &str = "/api/invite/{invite_key}";
pub const LINK_API_SESSION_ADD: &str = "/api/login";
pub const LINK_API_SESSION_GET_BY_SESSION_KEY: &str = "/api/profile";
pub const LINK_WEB_INDEX: &str = "/";
pub const LINK_WEB_LOGIN: &str = "/login";
pub const LINK_WEB_REGISTER: &str = "/login";
pub const LINK_WEB_CSS: &str = "/catsquad.css";
pub const LINK_WEB_WASM: &str = "/catsquad_bg.wasm";
pub const LINK_WEB_JS: &str = "/catsquad.js";
pub const LINK_WEB_FAVICON: &str = "/favicon.ico";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteAddRes {
    pub expires: u128,
}
