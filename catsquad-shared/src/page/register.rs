pub const LINK_WEB_REGISTER: &str = "/register";

pub const PATH_FRONT_END_REGISTER: &'static str = "/register";
// pub const PATH_FRONT_END_PARAM_STAGE: &'static str = "stage";
// pub const PATH_FRONT_END_PARAM_REG: &'static str = "reg";
// pub const PATH_FRONT_END_PARAM_TOKEN: &'static str = "token";

#[derive(Debug, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs)]
#[strum(serialize_all = "lowercase")]
pub enum RegisterPageParams {
    Stage,
    Token,
    Email,
}

#[derive(
    Debug, Default, Clone, PartialEq, PartialOrd, strum::EnumString, strum::Display, strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum RegisterPageStage {
    #[default]
    Invite,
    CheckEmail,
    Register,
}

pub fn link_relative_register() -> &'static str {
    LINK_WEB_REGISTER
}

pub fn link_relative_reg_check(email: impl AsRef<str>) -> String {
    format!(
        "{}?{}={}&{}={}",
        //
        PATH_FRONT_END_REGISTER,
        //
        RegisterPageParams::Stage.to_string(),
        RegisterPageStage::CheckEmail.to_string(),
        //
        RegisterPageParams::Email.to_string(),
        email.as_ref(),
    )
}

pub fn link_absolute_reg_finish(address: impl AsRef<str>, token: impl AsRef<str>) -> String {
    format!(
        "{}{}?{}={}&{}={}",
        //
        address.as_ref(),
        PATH_FRONT_END_REGISTER,
        //
        RegisterPageParams::Stage.to_string(),
        RegisterPageStage::Register.to_string(),
        //
        RegisterPageParams::Token.to_string(),
        token.as_ref(),
        // token.as_ref(),
    )
}
