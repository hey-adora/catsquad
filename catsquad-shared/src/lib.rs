mod api;
mod page;

pub const DEFAULT_GLOBAL_MAX_UPLOAD_SIZE: usize = 1000000000; // 1GB i think
pub const MAX_STORAGE_PER_FILE: u64 = 1024 * 1000 * 30; // 30MB
pub const MAX_STORAGE: u64 = 1024 * 1000 * 1000 * 2; // 2GB
pub const SUPPORTED_FILE_EXTENSIONS: &[&str] = &["ico", "svg", "jpg", "jpeg", "png", "webp"];
pub const MAX_POST_DESCRIPTION_LENGTH: usize = 2000;
pub const MAX_POST_COMMENT_LENGTH: usize = 2000;
pub const MAX_POST_TAGS_LENGTH: usize = 2000;
pub const MAX_POST_TITLE_LENGTH: usize = 120;
pub const MAX_USERNAME_LENGTH: usize = 32;
pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MIN_PASSWORD_LENGTH: usize = 12;
pub const MAX_PASSWORD_LENGTH: usize = 100;

pub use api::Order;
pub use api::TimeRange;
pub use api::ToForm;
pub use api::comment_add::*;
pub use api::comment_remove::*;
pub use api::comment_search::*;
pub use api::comment_update_text::*;
pub use api::email_change_add::*;
pub use api::email_change_update_cancel::*;
pub use api::email_change_update_current_confirm::*;
pub use api::email_change_update_finish::*;
pub use api::email_change_update_new_add::*;
pub use api::email_change_update_new_confirm::*;
pub use api::invite_add::*;
pub use api::invite_get_by_key::*;
pub use api::password_change_add::*;
pub use api::password_change_update_confirm::*;
pub use api::post_add::*;
pub use api::post_file_get_by_hash::*;
pub use api::post_get_by_key::*;
pub use api::post_like_add::*;
pub use api::post_like_get_post::*;
pub use api::post_like_remove::*;
pub use api::post_remove::*;
pub use api::post_search::*;
pub use api::post_update_description::*;
pub use api::post_update_file_add::*;
pub use api::post_update_file_remove::*;
pub use api::post_update_state::*;
pub use api::post_update_tags::*;
pub use api::post_update_title::*;
pub use api::session_add::*;
pub use api::session_delete::*;
pub use api::user_add::*;
pub use api::user_get_by_session_key::*;
pub use api::user_update_username::*;

pub use page::assets::*;
pub use page::index::*;
pub use page::login::*;
pub use page::post::*;
pub use page::register::*;
pub use page::settings::*;
pub use page::upload::*;

fn serde_from_u128<S: serde::Serializer>(v: &u128, serializer: S) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    let v = v.to_string();
    v.serialize(serializer)
}

fn serde_to_u128<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u128, D::Error> {
    use serde::Deserialize;
    String::deserialize(deserializer).map(|v| u128::from_str_radix(&v, 10).unwrap_or_default())
}
