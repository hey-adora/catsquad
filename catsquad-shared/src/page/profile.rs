use std::fmt::Display;

pub const LINK_WEB_PROFILE: &str = "/u/{username}";
pub const MATCH_CONTAINS: &str = "/u/"; // used in CurrentPage hook

pub fn link_relative_profile(username: impl Display) -> String {
    format!("/u/{username}")
}

pub fn link_relative_profile_search(username: impl Display, tags: impl Display) -> String {
    format!("/u/{}?tags={}", username, tags)
}
