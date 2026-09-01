pub mod comment_add;
pub mod comment_remove;
pub mod comment_search;
pub mod comment_update_text;
pub mod email_change_add;
pub mod email_change_resend;
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
pub mod post_file_get_by_hash;
pub mod post_get_by_key;
pub mod post_like_add;
pub mod post_like_get_post;
pub mod post_like_remove;
pub mod post_remove;
pub mod post_search;
pub mod post_update_description;
pub mod post_update_file_add;
pub mod post_update_file_remove;
pub mod post_update_state;
pub mod post_update_tags;
pub mod post_update_title;
pub mod session_add;
pub mod session_remove;
pub mod test_backdoor_email_sent_get_all;
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
    Less,
    LessOrEqual,
    More,
    MoreOrEqual,
}

impl From<u8> for TimeRange {
    fn from(value: u8) -> Self {
        match value {
            0 => TimeRange::None,
            1 => TimeRange::Less,
            2 => TimeRange::LessOrEqual,
            3 => TimeRange::More,
            4 => TimeRange::MoreOrEqual,
            v => panic!("invalid range {v}, valid: 0,1,2,3,4"),
        }
    }
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

impl From<bool> for Order {
    fn from(value: bool) -> Self {
        match value {
            false => Self::OneTwoThree,
            true => Self::ThreeTwoOne,
        }
    }
}

// #[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub enum Method {
//     Get,
//     Post,
// }
