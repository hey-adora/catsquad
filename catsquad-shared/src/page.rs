// pub const LINK_API_INVITE_GET_BY_KEY: &str = "/api/invite/{invite_key}";
// pub const LINK_API_SESSION_ADD: &str = "/api/login";
pub mod assets;
pub mod index;
pub mod login;
pub mod post;
pub mod register;
pub mod settings;
pub mod upload;

pub const MODAL_QUERY_PARAM_NAME: &'static str = "modal_confirm";

#[derive(
    Default, Debug, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum ModalQueryParamValue {
    #[default]
    None,
    Enabled,
}

pub fn modal_query_params() -> &'static str {
    "modal_confirm=enabled"
}

// pub const MODAL_QUERY_PARAM_STATE: &'static str = "enabled";
// #[derive(Debug, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs)]
// #[strum(serialize_all = "lowercase")]
// pub enum ModalConfirmStage {
//     ,
//     Token,
// }

// pub const PATH_FRONT_END_REGISTER: &'static str = "/register";
