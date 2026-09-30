use std::fmt::Display;

pub const LINK_WEB_PROFILE: &str = "/u/{username}";
pub const MATCH_PROFILE: &str = "/u/"; // used in CurrentPage hook
pub const MATCH_PROFILE_PAWS: &str = "tab=paws"; // used in CurrentPage hook
pub const MATCH_PROFILE_SUPPORT: &str = "tab=support"; // used in CurrentPage hook
pub const MATCH_PROFILE_ABOUTME: &str = "tab=aboutme"; // used in CurrentPage hook

pub fn link_relative_profile_gallery(username: impl Display) -> String {
    format!("/u/{username}")
}

pub fn link_relative_profile_paws(username: impl Display) -> String {
    format!("/u/{username}?tab=paws")
}

pub fn link_relative_profile_support(username: impl Display) -> String {
    format!("/u/{username}?tab=support")
}

pub fn link_relative_profile_aboutme(username: impl Display) -> String {
    format!("/u/{username}?tab=aboutme")
}

pub fn link_relative_profile_search(username: impl Display, tags: impl Display) -> String {
    format!("/u/{}?tags={}", username, tags)
}

pub fn link_relative_profile_paws_search(username: impl Display, tags: impl Display) -> String {
    format!("/u/{}?tab=paws&tags={}", username, tags)
}
