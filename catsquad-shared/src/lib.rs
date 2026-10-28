mod api;
mod page;
mod uuid;

pub const DEFAULT_GLOBAL_MAX_UPLOAD_SIZE: usize = 1000000000; // 1GB i think
pub const MAX_STORAGE_PER_FILE: u32 = 1024 * 1000 * 30; // 30MB
pub const MAX_STORAGE: u32 = 1024 * 1000 * 1000 * 2; // 2GB
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["ico", "svg", "jpg", "jpeg", "png", "webp"];
pub const MAX_POST_IMAGES_COUNT: usize = 10000;
pub const MAX_POST_DESCRIPTION_LENGTH: usize = 2000;
pub const MAX_POST_COMMENT_LENGTH: usize = 2000;
pub const MAX_POST_TAGS_LENGTH: usize = 2000;
pub const MAX_POST_TITLE_LENGTH: usize = 120;
pub const MAX_EMAIL_LENGTH: usize = 100;
pub const MIN_EMAIL_LENGTH: usize = 3;
pub const MAX_ABOUTME_LENGTH: usize = 2000;
pub const MAX_SUPPORT_LENGTH: usize = 2000;
pub const MAX_USERNAME_LENGTH: usize = 32;
pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MIN_PASSWORD_LENGTH: usize = 12;
pub const MAX_PASSWORD_LENGTH: usize = 100;
pub const FILE_IMAGE_THUMBNAIL_EXTENSION: &'static str = "webp";
pub const FILE_IMAGE_THUMBNAIL_SIZE: u32 = 1280;
pub const MAX_PFP_SIZE_BYTES: u32 = 1024 * 1000 * 2; // 2MB probably
pub const ADMIN_ROLE_NAME: &'static str = "admin";
pub type Hash = i64;

use anyhow::anyhow;
pub use api::Order;
pub use api::TimeRange;
pub use api::ToForm;
pub use api::comment_add::*;
pub use api::comment_remove::*;
pub use api::comment_search::*;
pub use api::comment_update_text::*;
pub use api::email_change_add::*;
pub use api::email_change_get_by_id::*;
pub use api::email_change_resend::*;
pub use api::email_change_update_cancel::*;
pub use api::email_change_update_current_confirm::*;
pub use api::email_change_update_finish::*;
pub use api::email_change_update_new_add::*;
pub use api::email_change_update_new_confirm::*;
pub use api::file_image::*;
pub use api::follow_add::*;
pub use api::follow_remove::*;
pub use api::invite_add::*;
pub use api::invite_get_by_key::*;
pub use api::post_add::*;
pub use api::post_get_by_key::*;
pub use api::post_image_bytes_get_by_hash::*;
pub use api::post_image_status_get_by_hash::*;
pub use api::post_like_add::*;
pub use api::post_like_get_post::*;
pub use api::post_like_remove::*;
pub use api::post_remove::*;
pub use api::post_search::*;
pub use api::post_thumbnail_bytes_get_by_hash::*;
pub use api::post_update_description::*;
pub use api::post_update_image_add::*;
pub use api::post_update_image_ord::*;
pub use api::post_update_image_remove::*;
pub use api::post_update_state::*;
pub use api::post_update_tags::*;
pub use api::post_update_title::*;
pub use api::role_add::*;
pub use api::role_remove::*;
pub use api::session_add::*;
pub use api::session_remove::*;
pub use api::test_backdoor_email_sent_get_all::*;
pub use api::token_password_change_add::*;
pub use api::token_user_remove_add::*;
pub use api::user_add::*;
pub use api::user_get_by_session_key::*;
pub use api::user_get_by_username::*;
pub use api::user_pfp_get_bytes::*;
pub use api::user_remove_by_token::*;
pub use api::user_search::*;
pub use api::user_update_aboutme::*;
pub use api::user_update_password_by_token::*;
pub use api::user_update_pfp::*;
pub use api::user_update_support::*;
pub use api::user_update_username::*;

pub use page::MODAL_QUERY_PARAM_NAME;
pub use page::ModalQueryParamValue;
pub use page::admin::*;
pub use page::assets::*;
pub use page::followed::*;
pub use page::index::*;
pub use page::login::*;
pub use page::modal_query_params;
pub use page::post::*;
pub use page::profile::*;
pub use page::register::*;
pub use page::settings::*;
pub use page::upload::*;

pub use uuid::*;

// pub fn get_time_ns() -> u128 {
//     use std::time::{SystemTime, UNIX_EPOCH};
//     let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
//     time.as_nanos()
// }

// pub fn get_time_micro() -> u128 {
//     use std::time::{SystemTime, UNIX_EPOCH};
//     let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
//     time.as_micros()
// }

fn serde_from_role_value<S: serde::Serializer>(
    v: &Vec<RoleValue>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;

    let mut output = String::new();
    let mut iter = v.into_iter();

    if let Some(v) = iter.next() {
        output.push_str(v.as_str());
    }

    for v in iter {
        output.push(',');
        output.push_str(v.as_str());
    }

    output.serialize(serializer)
}

fn serde_to_role_value<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<RoleValue>, D::Error> {
    use serde::Deserialize;

    String::deserialize(deserializer).and_then(|v| {
        v.split(',')
            .into_iter()
            .map(|v| RoleValue::try_from(v))
            .collect::<anyhow::Result<Vec<RoleValue>>>()
            .map_err(serde::de::Error::custom)
    })
}

fn serde_from_uuid<S: serde::Serializer>(v: &Uuid, serializer: S) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    let v = uuid_to_str(*v);
    v.serialize(serializer)
}

fn serde_to_uuid<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Uuid, D::Error> {
    use serde::Deserialize;
    String::deserialize(deserializer).map(|v| str_to_uuid(v))
}

fn serde_from_u128<S: serde::Serializer>(v: &u128, serializer: S) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    let v = v.to_string();
    v.serialize(serializer)
}

fn serde_to_u128<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u128, D::Error> {
    use serde::Deserialize;
    String::deserialize(deserializer).map(|v| u128::from_str_radix(&v, 10).unwrap_or_default())
}

fn serde_from_option_u128<S: serde::Serializer>(
    v: &Option<u128>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    let v = v.map(|v| v.to_string());
    v.serialize(serializer)
}

fn serde_to_option_u128<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u128>, D::Error> {
    use serde::Deserialize;
    <Option<String>>::deserialize(deserializer)
        .map(|v| v.map(|v| u128::from_str_radix(&v, 10).unwrap_or_default()))
}
