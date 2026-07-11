use catsquad_log::prelude::*;

pub fn validate_email<S: AsRef<str>>(email: S) -> Result<(), String> {
    let mut errors = String::new();
    let email = email.as_ref();

    if email.is_empty() {
        errors += "email cannot be empty\n";
    }

    if !email.contains('@') {
        errors += "email must contain '@'\n";
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
fn test_validator_validate_email() {
    assert!(validate_email("").is_err());
    assert!(validate_email(" ").is_err());
    assert!(validate_email("a").is_err());
    assert!(validate_email("a@").is_ok());
}

pub fn validate_username(username: impl AsRef<str>) -> Result<(), String> {
    let mut errors = String::new();
    let username = username.as_ref();
    match username.len() {
        len if len < 3 => errors += "username must be at least 3 characters length\n",
        len if len > 32 => errors += "username must be shorter than 33 characters length\n",
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

    if password_len < 12 {
        errors += "password must be at least 12 characters long\n";
    }
    if password_len > 128 {
        errors += "password must be shorter than 129 characters\n";
    }

    let mut contains_number = false;
    let mut contains_symbol = false;
    let mut contains_capital = false;
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
}
