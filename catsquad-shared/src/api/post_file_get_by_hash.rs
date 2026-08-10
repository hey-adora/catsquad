use catsquad_log::prelude::*;

pub const LINK_API_POST_FILE_GET_BY_HASH: &str = "/api/post/{post_key}/file/{file_hash}";

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct StorageParams {
    pub post_key: String,
    pub file_hash: String,
}

pub fn link_relative_img(post_key: impl AsRef<str>, file_hash: impl AsRef<str>) -> String {
    format!(
        "/api/post/{}/file/{}",
        post_key.as_ref(),
        file_hash.as_ref()
    )
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
