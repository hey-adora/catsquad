pub const LINK_WEB_POST: &str = "/p/{id}";

pub fn link_relative_post(post_id: i64) -> String {
    format!("/p/{}", post_id)
}
