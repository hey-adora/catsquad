mod server;

mod api;
mod api_config;
mod assets;
mod state;

pub mod auth;
pub mod utils;
// pub mod validation;
// pub mod web;

#[cfg(test)]
use std::os::unix::fs::MetadataExt;

pub use catsquad_db::id_to_string;
pub use server::server;

#[cfg(any(test, feature = "test_server"))]
mod test_server;
#[cfg(any(test, feature = "test_server"))]
pub use test_server::TestServer;

#[cfg(test)]
pub async fn get_file_size(file_path: impl AsRef<std::path::Path>) -> u64 {
    let file = tokio::fs::metadata(file_path).await.unwrap();
    file.size()
}

#[cfg(test)]
pub async fn get_file_hash_for_testing_by_path(file_path: impl AsRef<str>) -> String {
    let file = tokio::fs::read(file_path.as_ref()).await.unwrap();
    get_file_hash_for_testing(&file)
}

#[cfg(test)]
pub fn get_file_hash_for_testing(file: &[u8]) -> String {
    use std::hash::Hasher;
    // let file = tokio::fs::read(file_path.as_ref()).await.unwrap();
    // let mut hasher = GxBuildHasher::default();
    let mut hasher = std::hash::DefaultHasher::default();
    // let mut hasher = GxHasher::with_seed(0);
    hasher.write(&file);
    hasher.finish().to_string()
}
