mod server;

mod api;
mod api_config;
mod state;

pub mod utils;
pub mod validation;
pub mod web;

pub use server::server;

#[cfg(test)]
mod test_server;
#[cfg(test)]
pub use test_server::TestServer;

pub const MAX_STORAGE_PER_FILE: u64 = 1024 * 30; // 30MB
pub const MAX_STORAGE: u64 = 1024 * 1000 * 2; // 2GB
pub const SUPPORTED_FILE_EXTENSIONS: &[&str] = &["ico", "svg", "jpg", "jpeg", "png", "webp"];
pub const MAX_POST_DESCRIPTION_LENGTH: usize = 2000;
pub const MAX_POST_COMMENT_LENGTH: usize = 2000;
pub const MAX_POST_TAGS_LENGTH: usize = 2000;
pub const MAX_POST_TITLE_LENGTH: usize = 120;
