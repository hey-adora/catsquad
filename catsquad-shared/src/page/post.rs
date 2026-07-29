pub const LINK_WEB_POST: &str = "/c/{id}";

pub fn link_relative_post(post_key: impl AsRef<str>) -> String {
    format!("/c/{}", post_key.as_ref())
}
