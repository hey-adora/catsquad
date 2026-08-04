pub mod email_change_add;
pub mod email_change_update_cancel;
pub mod email_change_update_current_confirm;
pub mod email_change_update_finish;
pub mod email_change_update_new_add;
pub mod email_change_update_new_confirm;
pub mod invite_add;
pub mod invite_get_by_key;
pub mod password_change_add;
pub mod password_change_update_confirm;
pub mod post_add;
pub mod post_get_by_key;
pub mod post_search;
pub mod post_update_description;
pub mod post_update_file_add;
pub mod post_update_file_remove;
pub mod post_update_state;
pub mod post_update_tags;
pub mod post_update_title;
pub mod session_add;
pub mod session_delete;
pub mod user_add;
pub mod user_get_by_session_key;
pub mod user_update_username;
// pub mod invite_get_email_by_key;

pub trait ToForm {
    fn to_form(&self) -> Result<String, anyhow::Error>;
}

impl<T: serde::Serialize> ToForm for T {
    fn to_form(&self) -> Result<String, anyhow::Error> {
        to_form(self)
    }
}

pub fn to_form(data: impl serde::Serialize) -> Result<String, anyhow::Error> {
    serde_urlencoded::to_string(data).map_err(|v| v.into())
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
pub enum TimeRange {
    #[default]
    None,
    Less(u128),
    LessOrEqual(u128),
    More(u128),
    MoreOrEqual(u128),
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
pub enum Order {
    #[default]
    OneTwoThree,
    ThreeTwoOne,
}

// #[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub enum Method {
//     Get,
//     Post,
// }
