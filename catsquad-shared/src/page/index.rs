pub const LINK_WEB_INDEX: &str = "/";

pub fn link_relative_index_search(tags: impl AsRef<str>) -> String {
    format!("/?tags={}", tags.as_ref())
}
