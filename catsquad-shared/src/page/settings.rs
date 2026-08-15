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
        "/settings?{}={}",
        SettingsPageParams::Stage,
        SettingsPageStage::ChangeEmailCurrentAdd
    )
}

#[derive(Debug, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs)]
#[strum(serialize_all = "lowercase")]
pub enum SettingsPageParams {
    Stage,
    Token,
    Email,
}

#[derive(
    Debug, Default, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum SettingsPageStage {
    #[default]
    None,
    UsernameChange,
    ChangeEmailCurrentAdd,
    ChangeEmailCurrentConfirm,
    ChangeEmailNewAdd,
    ChangeEmailNewConfirm,
    ChangeEmailFinish,
}
