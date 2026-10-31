use crate::{Uuid, serde_from_uuid, serde_to_uuid};

pub const LINK_API_SESSION_ADD: &str = "/api/login";

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct SessionRes {
//     // #[serde(serialize_with = "serde_from_uuid", deserialize_with = "serde_to_uuid")]
//     // pub token: Uuid,
//     pub username: String,
//     pub email: String,
//     pub created_at: u64,
// }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SessionAddReq {
    pub email: String,
    pub password: String,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum SessionAddErr {
    #[error("email or password is wrong")]
    InvalidCredentials,

    #[error("bad request {0}")]
    BadRequest(String),

    #[default]
    #[error("internal server err")]
    InternalServer,
}

pub fn extract_auth_token_plain(input: impl AsRef<str>) -> Option<String> {
    let input = input.as_ref();
    let input_len = input.len();
    let mut i = input.chars();

    let mut start = 0;
    let mut end;
    // let mut stage = 0_usize;

    while let Some(c) = i.next() {
        start += 1;
        if c == ' ' {
            break;
        }
    }

    end = start;

    while let Some(c) = i.next() {
        let valid_char = (c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z');
        if !valid_char {
            break;
        }
        end += 1;
    }

    let is_not_out_of_index = end <= input_len && start < end;
    if !is_not_out_of_index {
        return None;
    }

    Some(input[start..end].to_string())
}

#[cfg(test)]
#[test]
fn test_extract_auth_token_plain() {
    use catsquad_log::prelude::*;

    init_log();

    let token =
        extract_auth_token_plain("authorization=Bearer 34JSuIfanzy1UDwwCLh3c; HttpOnly; Secure")
            .unwrap();
    assert_eq!(token, "34JSuIfanzy1UDwwCLh3c");
}
