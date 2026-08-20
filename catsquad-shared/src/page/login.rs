use std::fmt::Display;

pub const LINK_WEB_LOGIN: &str = "/login";

#[derive(Debug, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum LoginPageParams {
    PageStage,
    PssResetStage,
    Token,
    Email,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum LoginPageStage {
    #[default]
    None,
    PssReset,
    // PssChangeCheck,
    // PssChangeConfirm,
    // PssChangeFinish,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum PasswordResetStage {
    #[default]
    Add,
    Check,
    Confirm,
    Finished,
}

pub fn link_relative_login() -> &'static str {
    LINK_WEB_LOGIN
}

pub fn link_relative_login_password_reset_send() -> String {
    format!(
        "{LINK_WEB_LOGIN}?{}={}",
        LoginPageParams::PageStage,
        LoginPageStage::PssReset,
    )
}

pub fn link_relative_login_password_reset_check(email: impl Display) -> String {
    format!(
        "{LINK_WEB_LOGIN}?{}={}&{}={}&{}={}",
        LoginPageParams::PageStage,
        LoginPageStage::PssReset,
        LoginPageParams::PssResetStage,
        PasswordResetStage::Check,
        LoginPageParams::Email,
        email,
    )
}

pub fn link_relative_login_password_reset_confirm(pss_change_key: impl Display) -> String {
    format!(
        "{LINK_WEB_LOGIN}?{}={}&{}={}&{}={}",
        LoginPageParams::PageStage,
        LoginPageStage::PssReset,
        LoginPageParams::PssResetStage,
        PasswordResetStage::Confirm,
        LoginPageParams::Token,
        pss_change_key,
    )
}

pub fn link_relative_login_password_reset_finished(email: impl Display) -> String {
    format!(
        "{LINK_WEB_LOGIN}?{}={}&{}={}&{}={}",
        LoginPageParams::PageStage,
        LoginPageStage::PssReset,
        LoginPageParams::PssResetStage,
        PasswordResetStage::Finished,
        LoginPageParams::Email,
        email,
    )
}

// pub fn link_login() -> &'static str {
//     LINK_WEB_LOGIN
// }

// pub fn link_login_form_password_send() -> String {
//     link_login_form_password(
//         ChangePasswordFormStage::Send,
//         None::<String>,
//         None::<String>,
//     )
// }

// pub fn link_login_form_password_confirm(
//     email: impl Into<String>,
//     confirm_key: impl Into<String>,
// ) -> String {
//     link_login_form_password(
//         ChangePasswordFormStage::Confirm,
//         Some(email),
//         Some(confirm_key),
//     )
// }

// pub fn link_login_form_password(
//     stage: ChangePasswordFormStage,
//     email: Option<impl Into<String>>,
//     confirm_key: Option<impl Into<String>>,
// ) -> String {
//     format!(
//         "{}{}",
//         link_login(),
//         query_form_password(stage, email, confirm_key),
//     )
// }

// pub fn query_form_password(
//     stage: ChangePasswordFormStage,
//     email: Option<impl Into<String>>,
//     confirm_key: Option<impl Into<String>>,
// ) -> String {
//     format!(
//         "?{}={}{}{}",
//         ChangePasswordQueryFields::FormStage,
//         stage,
//         match email {
//             Some(v) => format!("&{}={}", ChangePasswordQueryFields::Email, v.into()),
//             None => "".to_string(),
//         },
//         match confirm_key {
//             Some(v) => format!("&{}={}", ChangePasswordQueryFields::Token, v.into()),
//             None => "".to_string(),
//         },
//     )
// }

// #[derive(
//     Debug,
//     Default,
//     Clone,
//     PartialEq,
//     PartialOrd,
//     strum::EnumString,
//     strum::Display,
//     strum::EnumIter,
//     strum::EnumIs,
// )]
// #[strum(serialize_all = "PascalCase")]
// pub enum ChangePasswordBtnStage {
//     #[default]
//     None,
//     Send,
//     ReSend,
//     Confirm,
// }

// // #[derive(
// //     Debug,
// //     Default,
// //     Clone,
// //     PartialEq,
// //     PartialOrd,
// //     strum::EnumString,
// //     strum::Display,
// //     strum::EnumIter,
// //     strum::EnumIs,
// // )]
// // #[strum(serialize_all = "lowercase")]
// // pub enum ChangePasswordFormStage {
// //     #[default]
// //     None,
// //     Send,
// //     Check,
// //     Confirm,
// //     Finish,
// // }
