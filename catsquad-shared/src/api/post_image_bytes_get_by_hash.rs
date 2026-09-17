use crate::i64_to_str;

pub const LINK_API_POST_IMAGE_BYTES_GET_BY_HASH: &str = "/api/post/{post_id}/file/{file_hash}";

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct PostImageBytesGetByHashParams {
    pub post_id: i64,
    pub file_hash: i64,
}

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum PostImageBytesGetByHashErr {
    #[error("post file not found")]
    FileNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[default]
    #[error("internal server err")]
    InternalServerErr,
}

pub fn link_relative_post_image_bytes_get_by_hash(post_id: i64, file_hash: i64) -> String {
    // let file_hash = i64_to_str(file_hash);
    format!("/api/post/{}/file/{}", post_id, file_hash)
}

// pub fn validate_storage_path(storage_path: impl AsRef<str>) -> Result<(), String> {
//     let mut errors = String::new();
//     let value = storage_path.as_ref();

//     if value.is_empty() {
//         errors += "path cant be empty";
//         trace!("errors {errors}");
//         return Err(errors);
//     }

//     for c in value.chars() {
//         let valid_char = (c >= '0' && c <= '9') || (c >= 'a' && c <= 'z');
//         if !valid_char {
//             errors += "invalid path\n";
//             break;
//         }
//     }

//     // if !email.contains('@') {
//     //     errors += "email must contain '@'\n";
//     // }

//     if errors.is_empty() {
//         Ok(())
//     } else {
//         let _ = errors.pop();
//         trace!("errors {errors}");
//         Err(errors)
//     }
// }

// #[test]
// fn test_validate_storage_path() {
//     assert!(validate_storage_path("").is_err());
//     assert!(validate_storage_path("0123456789").is_ok());
//     assert!(validate_storage_path("/0123456789").is_err());
//     assert!(validate_storage_path("0123456789:").is_err());
//     assert!(validate_storage_path("0123456789abcdefghijklmnopqrstuvwxyz").is_ok());
//     assert!(validate_storage_path("0123456789abcdefghijklmnopqrstuvwxyzA").is_err());
//     // assert!(validate_storage_path("0").is_ok());
//     // assert!(validate_storage_path(" ").is_err());
//     // assert!(validate_storage_path("a").is_err());
//     // assert!(validate_storage_path("a@").is_ok());
// }
