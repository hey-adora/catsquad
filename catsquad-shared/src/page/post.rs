pub const LINK_WEB_POST: &str = "/p/{id}";

pub fn link_relative_post(post_key: impl AsRef<str>) -> String {
    format!("/p/{}", post_key.as_ref())
}
