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

pub fn link_relative_settings_email_change_current_check_email() -> String {
    format!(
        "/settings?{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailCurrentCheckEmail,
    )
}

pub fn link_relative_settings_email_change_current_confirm(
    email_change_key: impl Into<String>,
    token: impl Into<String>,
) -> String {
    let email_change_key = email_change_key.into();
    let token = token.into();
    format!(
        "/settings?{}={}&{}={}&{}={}&{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::EmailChange,
        //
        SettingsPageParams::EmailChangeKey,
        email_change_key,
        //
        SettingsPageParams::EmailChangeStage,
        EmailCangeStage::ChangeEmailCurrentConfirm,
        //
        SettingsPageParams::Token,
        token,
    )
}

pub fn link_absolute_settings_email_change_current_confirm(
    host: Url,
    email_change_key: impl Into<String>,
    token: impl Into<String>,
) -> Result<Url, url::ParseError> {
    let relative = link_relative_settings_email_change_current_confirm(email_change_key, token);
    host.join(&relative)
}

#[derive(Debug, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs)]
#[strum(serialize_all = "lowercase")]
pub enum SettingsPageParams {
    Stage,
    Token,
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
    ChangeEmailNewConfirm,
    ChangeEmailFinish,
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
