use std::fmt::Display;

use url::Url;

pub const LINK_WEB_SETTINGS: &str = "/settings";

pub fn link_relative_settings() -> &'static str {
    LINK_WEB_SETTINGS
}

pub fn link_relative_settings_username_change() -> String {
    format!(
        "/settings?{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::UsernameChange
    )
}

pub fn link_relative_settings_email_change_current_add() -> String {
    format!(
        "/settings?{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailCurrentAdd,
    )
}

pub fn link_relative_settings_email_change_current_check_email(
    email_change_key: impl Display,
) -> String {
    format!(
        "/settings?{}={}&{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailCurrentCheckEmail,
        //
        SettingsPageParams::EmailChangeKey,
        email_change_key,
    )
}

pub fn link_relative_settings_email_change_current_confirm(
    email_change_key: impl Display,
    token: impl Display,
) -> String {
    format!(
        "/settings?{}={}&{}={}&{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailCurrentConfirm,
        //
        SettingsPageParams::EmailChangeKey,
        email_change_key,
        //
        SettingsPageParams::Token,
        token,
    )
}

pub fn link_absolute_settings_email_change_current_confirm(
    host: Url,
    email_change_key: impl Display,
    token: impl Display,
) -> Result<Url, url::ParseError> {
    let relative = link_relative_settings_email_change_current_confirm(email_change_key, token);
    host.join(&relative)
}

pub fn link_relative_settings_email_change_new_add(email_change_key: impl Display) -> String {
    format!(
        "/settings?{}={}&{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeKey,
        email_change_key,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailNewAdd,
    )
}

pub fn link_relative_settings_email_change_new_check_email(
    email_change_key: impl Display,
    new_email: impl Display,
) -> String {
    format!(
        "/settings?{}={}&{}={}&{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailNewCheckEmail,
        //
        SettingsPageParams::NewEmail,
        new_email,
        //
        SettingsPageParams::EmailChangeKey,
        email_change_key,
    )
}

pub fn link_relative_settings_email_change_new_confirm(
    email_change_key: impl Display,
    token: impl Display,
) -> String {
    format!(
        "/settings?{}={}&{}={}&{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailNewConfirm,
        //
        SettingsPageParams::Token,
        token,
        //
        SettingsPageParams::EmailChangeKey,
        email_change_key,
    )
}

pub fn link_absolute_settings_email_change_new_confirm(
    host: Url,
    email_change_key: impl Display,
    token: impl Display,
) -> Result<Url, url::ParseError> {
    let relative = link_relative_settings_email_change_new_confirm(email_change_key, token);
    host.join(&relative)
}

pub fn link_relative_settings_email_change_finish(email_change_key: impl Display) -> String {
    format!(
        "/settings?{}={}&{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailFinish,
        //
        SettingsPageParams::EmailChangeKey,
        email_change_key,
    )
}

pub fn link_relative_settings_email_change_finished(email_change_key: impl Display) -> String {
    format!(
        "/settings?{}={}&{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailFinished,
        //
        SettingsPageParams::EmailChangeKey,
        email_change_key,
    )
}

pub fn link_relative_settings_email_change_canceled() -> String {
    format!(
        "/settings?{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailCanceled,
    )
}

#[derive(Debug, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs)]
#[strum(serialize_all = "lowercase")]
pub enum SettingsPageParams {
    Stage,
    Token,
    NewEmail,
    EmailChangeKey,
    EmailChangeStage,
}

#[derive(
    Default, Debug, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum EmailCangeStage {
    #[default]
    ChangeEmailCurrentAdd,
    ChangeEmailCurrentCheckEmail,
    ChangeEmailCurrentConfirm,
    ChangeEmailNewAdd,
    ChangeEmailNewCheckEmail,
    ChangeEmailNewConfirm,
    ChangeEmailFinish,
    ChangeEmailFinished,
    ChangeEmailCanceled,
}

#[derive(
    Debug, Default, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum SettingsPageStage {
    #[default]
    None,
    UsernameChange,
    EmailChange,
}
