pub const LINK_WEB_ADMIN: &str = "/admin";
pub const MATCH_ADMIN: &str = "/admin";
pub const MATCH_ADMIN_USERS: &str = "tab=users"; // used in CurrentPage hook
pub const MATCH_ADMIN_ROLES: &str = "tab=roles"; // used in CurrentPage hook

pub fn link_relative_admin() -> &'static str {
    "/admin"
}

pub fn link_relative_admin_users() -> &'static str {
    "/admin?tab=users"
}

pub fn link_relative_admin_roles() -> &'static str {
    "/admin?tab=roles"
}
