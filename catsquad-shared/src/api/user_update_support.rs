use crate::MAX_SUPPORT_LENGTH;
use catsquad_log::prelude::*;

pub const LINK_API_USER_UPDATE_SUPPORT: &str = "/api/user_update_support";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateSupportReq {
    pub new_support: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserUpdateSupportErr {
    #[error("support is invalid {0}")]
    Invalid(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}

pub fn validate_user_support(support: impl AsRef<str>) -> Result<(), String> {
    let mut errors = String::new();
    let input = support.as_ref();

    if input.len() > MAX_SUPPORT_LENGTH {
        errors += "max support length is 2000 characters\n";
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}
