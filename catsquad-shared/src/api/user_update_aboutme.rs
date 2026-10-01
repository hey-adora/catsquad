use crate::MAX_ABOUTME_LENGTH;
use catsquad_log::prelude::*;

pub const LINK_API_USER_UPDATE_ABOUTME: &str = "/api/user_update_aboutme";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserUpdateAboutmeReq {
    pub new_aboutme: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserUpdateAboutmeErr {
    #[error("aboutme is invalid {0}")]
    Invalid(String),

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}

pub fn validate_user_aboutme(aboutme: impl AsRef<str>) -> Result<(), String> {
    let mut errors = String::new();
    let input = aboutme.as_ref();

    if input.len() > MAX_ABOUTME_LENGTH {
        errors += "max aboutme length is 2000 characters\n";
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}
