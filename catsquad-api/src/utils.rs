use axum::http::{HeaderMap, header::SET_COOKIE};
use catsquad_log::prelude::*;

const COOKIE_PREFIX: &'static str = "Bearer ";
const COOKIE_PREFIX_FULL: &'static str = "authorization=Bearer ";
const COOKIE_POSTFIX: &'static str = "; HttpOnly; Secure";
const COOKIE_DELETED: &'static str =
    "authorization=Bearer DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT";

pub fn create_auth_cookie(token: impl AsRef<str>) -> HeaderMap {
    let cookie = format!("{}{}{}", COOKIE_PREFIX_FULL, token.as_ref(), COOKIE_POSTFIX);
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    trace!("set auth cookie {}", cookie);
    headers
}
