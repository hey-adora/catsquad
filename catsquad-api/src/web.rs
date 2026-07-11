pub const PATH_FRONT_END_REGISTER: &'static str = "/register";
pub const PATH_FRONT_END_PARAM_STAGE: &'static str = "stage";
pub const PATH_FRONT_END_PARAM_REG: &'static str = "reg";
pub const PATH_FRONT_END_PARAM_TOKEN: &'static str = "token";

pub fn link_absolute_reg_finish(address: impl AsRef<str>, token: impl AsRef<str>) -> String {
    format!(
        "{}{}?{}={}&{}={}",
        //
        address.as_ref(),
        PATH_FRONT_END_REGISTER,
        //
        PATH_FRONT_END_PARAM_STAGE,
        PATH_FRONT_END_PARAM_REG,
        //
        PATH_FRONT_END_PARAM_TOKEN,
        token.as_ref(),
    )
}
