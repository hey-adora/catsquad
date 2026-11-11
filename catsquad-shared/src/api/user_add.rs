use std::fmt::Display;

use crate::{
    MAX_EMAIL_LENGTH, MAX_PASSWORD_LENGTH, MAX_USERNAME_LENGTH, MIN_EMAIL_LENGTH,
    MIN_PASSWORD_LENGTH, MIN_USERNAME_LENGTH, Uuid,
};
use anyhow::anyhow;
use catsquad_log::prelude::*;

pub const LINK_API_USER_ADD: &str = "/api/register";
pub const USER_STATE_ACTIVE: &str = "active";
pub const USER_STATE_SUSPENDED: &str = "suspended";

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct RedactedUserRes {
//     // pub key: i64,
//     pub username: String,
//     pub pfp_hash: i64,
//     pub pfp_proccesed: bool,
//     pub support: String,
//     pub aboutme: String,
//     pub created_at: u64,
// }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SensitiveUserRes {
    // pub key: i64,
    pub username: String,
    pub pfp_hash: i64,
    pub pfp_proccesed: bool,
    pub roles_ids: Vec<i64>,
    pub max_storage_bytes: u32, // TODO add max per image and add that to UI
    pub email: String,
    pub created_at: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserAddReq {
    pub username: String,
    pub password: String,
    #[serde(
        serialize_with = "crate::serde_from_uuid",
        deserialize_with = "crate::serde_to_uuid"
    )]
    pub invite_token: Uuid,
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum UserState {
    Active,
    Suspended,
}

impl UserState {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserState::Active => USER_STATE_ACTIVE,
            UserState::Suspended => USER_STATE_SUSPENDED,
        }
    }
}

impl Display for UserState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for UserState {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.as_str();
        TryFrom::<&str>::try_from(value)
    }
}

impl TryFrom<&str> for UserState {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            USER_STATE_SUSPENDED => Ok(Self::Suspended),
            USER_STATE_ACTIVE => Ok(Self::Active),
            _ => Err(anyhow!("database has invalid state")),
        }
    }
}

impl PartialEq<&str> for UserState {
    fn eq(&self, other: &&str) -> bool {
        let other = *other;
        let Ok(other) = UserState::try_from(other) else {
            return false;
        };
        *self == other
    }
}

impl PartialEq<String> for UserState {
    fn eq(&self, other: &String) -> bool {
        let other = other.as_str();
        let Ok(other) = UserState::try_from(other) else {
            return false;
        };
        *self == other
    }
}

impl PartialEq<UserState> for String {
    fn eq(&self, b: &UserState) -> bool {
        let a = self.as_str();
        let Ok(a) = UserState::try_from(a) else {
            return false;
        };
        a == *b
    }
}

impl PartialEq<UserState> for &str {
    fn eq(&self, b: &UserState) -> bool {
        let a = *self;
        let Ok(a) = UserState::try_from(a) else {
            return false;
        };
        a == *b
    }
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum UserAddErr {
    #[error("invalid input")]
    InvalidInput {
        username: Option<String>,
        password: Option<String>,
    },

    #[error("email is taken")]
    EmailIsTaken,

    #[error("username is taken")]
    UsernameIsTaken,

    #[error("invite not found")]
    InviteNotFound,

    #[error("invite already used")]
    InviteAlreadyUsed,

    #[error("invite expired")]
    InviteExpired,

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}

pub fn validate_email<S: AsRef<str>>(email: S) -> Result<(), String> {
    let mut errors = String::new();
    let email = email.as_ref();

    if email.is_empty() {
        errors += "email cannot be empty\n";
    }

    if !email.contains('@') {
        errors += "email must contain '@'\n";
    }

    match email.len() {
        len if len < MIN_EMAIL_LENGTH => errors += "min email length is 3 characters length\n",
        len if len > MAX_EMAIL_LENGTH => errors += "max email length is 100 characters\n",
        _ => {}
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}

#[test]
fn test_validate_email() {
    use rand::distr::SampleString;
    let long_email = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), MAX_EMAIL_LENGTH);
    let long_email = format!("{long_email}@heyadora.com");

    assert!(validate_email(long_email).is_err());
    assert!(validate_email("").is_err());
    assert!(validate_email(" ").is_err());
    assert!(validate_email("a").is_err());
    assert!(validate_email("a@").is_err());
    assert!(validate_email("a@.").is_ok());
}

pub fn validate_username(username: impl AsRef<str>) -> Result<(), String> {
    let mut errors = String::new();
    let username = username.as_ref();
    match username.len() {
        len if len < MIN_USERNAME_LENGTH => {
            errors += "min username length is 3 characters length\n"
        }
        len if len > MAX_USERNAME_LENGTH => errors += "max username length is 32 characters\n",
        _ => {}
    }
    let mut username_chars = username.chars();
    match username_chars.next() {
        Some(c) if c.is_alphabetic() => {}
        _ => errors += "username must start with alphabetic character\n",
    }
    for c in username_chars {
        if !(c.is_alphanumeric() || c == '_') {
            errors += "username must be alphanumeric\n";
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}

#[test]
fn test_validate_username() {
    use rand::distr::SampleString;

    assert!(validate_username("hey").is_ok());
    assert!(validate_username("hey%").is_err());
    assert!(validate_username("he").is_err());
    assert!(validate_username("00000000000000000000000000000000").is_err());
    assert!(validate_username("a0000000000000000000000000000000").is_ok());
    assert!(validate_username("a00000000000000000000000000000000").is_err());
}

pub fn validate_password(password: impl AsRef<str>) -> Result<(), String> {
    let mut errors = String::new();
    let password = password.as_ref();
    let password_len = password.len();

    if password_len < MIN_PASSWORD_LENGTH {
        errors += "min password length is 12 characters\n";
    }
    if password_len > MAX_PASSWORD_LENGTH {
        errors += "max password length is 128 characters\n";
    }

    let mut contains_number = false;
    let mut contains_symbol = false;
    let mut contains_capital = false;
    let mut contains_lowercase = false;
    for c in password.chars() {
        if c.is_numeric() {
            contains_number = true;
        }
        if !c.is_alphanumeric() {
            contains_symbol = true;
        }
        if c.is_uppercase() {
            contains_capital = true;
        }
        if c.is_lowercase() {
            contains_lowercase = true;
        }
    }

    if !contains_number {
        errors += "password must contain at least one number\n";
    }
    if !contains_symbol {
        errors += "password must contain at least one symbol\n";
    }
    if !contains_capital {
        errors += "password must contain at least one uppercase letter\n";
    }
    if !contains_lowercase {
        errors += "password must contain at least one lowercase letter\n";
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let _ = errors.pop();
        trace!("errors {errors}");
        Err(errors)
    }
}

#[test]
fn test_validate_password() {
    assert!(validate_password("password").is_err());
    assert!(validate_password("password123").is_err());
    assert!(validate_password("passw*rd123").is_err());
    assert!(validate_password("passw*rd1232").is_err());
    assert!(validate_password("passw*rD1232").is_ok());
    assert!(validate_password("PASSWD*RD1232").is_err());
}
