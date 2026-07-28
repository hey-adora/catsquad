use axum::{
    Json,
    extract::State,
    http::{
        HeaderMap, StatusCode,
        header::{self, COOKIE, SET_COOKIE},
    },
    response::IntoResponse,
};
use catsquad_db::{DbSessionGetByKeyErr, DbUser};
use catsquad_log::prelude::*;

use crate::state::AppState;

pub const ERR_MSG_COOKIE: &str = "no valid cookie";
pub const ERR_MSG_SESSION: &str = "no valid session";

const COOKIE_PREFIX: &'static str = "Bearer ";
pub const COOKIE_PREFIX_FULL: &'static str = "authorization=Bearer ";
const COOKIE_POSTFIX: &'static str = "; HttpOnly; Secure";
const COOKIE_DELETED: &'static str =
    "authorization=Bearer DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SessionKey(pub String);

impl ToString for SessionKey {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(thiserror::Error, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AuthErr {
    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("internal server err")]
    InternalServer,
}

pub fn verify_password<T: AsRef<[u8]>, S2: AsRef<str>>(
    password: T,
    hash: S2,
) -> anyhow::Result<()> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let password = password.as_ref();
    let hash = hash.as_ref();
    PasswordHash::new(hash)
        .and_then(|hash| Argon2::default().verify_password(password, &hash))
        .map_err(|err| err.into())
}

pub fn hash_password<S: Into<String>>(password: S) -> Result<String, argon2::password_hash::Error> {
    use argon2::{
        Argon2, PasswordHasher,
        password_hash::{
            SaltString,
            rand_core::{OsRng, RngCore},
        },
    };

    let rng = &mut OsRng;
    let mut bytes = [0u8; 10]; // 10 is salt length, bigger = slower = more secure
    rng.fill_bytes(&mut bytes);
    let salt = SaltString::encode_b64(&bytes)?;
    let argon2 = Argon2::default();
    let password = password.into();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

pub fn create_deleted_cookie() -> HeaderMap {
    let cookie = COOKIE_DELETED.to_string();
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers
}

pub fn create_auth_cookie(token: impl AsRef<str>) -> HeaderMap {
    let cookie = create_auth_cookie_str(token);
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    trace!("set auth cookie {}", cookie);
    headers
}

pub fn create_auth_cookie_str(token: impl AsRef<str>) -> String {
    format!("{}{}{}", COOKIE_PREFIX_FULL, token.as_ref(), COOKIE_POSTFIX)
}

pub async fn auth_optional_middleware(
    State(app_state): State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let result = {
        let headers = req.headers();
        check_auth(&app_state, &headers).await
    };

    match result {
        Ok((token, user)) => {
            let extensions = req.extensions_mut();
            extensions.insert(Some::<SessionKey>(SessionKey(token)));
            extensions.insert(Some::<DbUser>(user));
        }
        Err(_err) => {
            let extensions = req.extensions_mut();
            extensions.insert(None::<SessionKey>);
            extensions.insert(None::<DbUser>);
        }
    }
    next.run(req).await
}

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let result = {
        let headers = req.headers();
        // let jar = CookieJar::from_headers(headers);
        check_auth(&app_state, &headers).await
    };
    match result {
        Ok((token, user)) => {
            {
                let extensions = req.extensions_mut();
                extensions.insert(SessionKey(token));
                extensions.insert(user);
            }
            let response = next.run(req).await;
            return response;
        }
        Err(err) => {
            let status = StatusCode::UNAUTHORIZED;
            let result = Json(Err::<(), AuthErr>(err));
            return (status, result).into_response();
        }
    }
}

pub async fn check_auth(app: &AppState, headers: &HeaderMap) -> Result<(String, DbUser), AuthErr> {
    trace!("CHECKING AUTH");

    let token = auth_token_get(headers, header::COOKIE)
        .ok_or(AuthErr::Unauthorized(ERR_MSG_COOKIE.to_string()))?;

    trace!("CHECKING AUTH SESSION");
    let session = app
        .db
        .session_get_by_key(token.clone())
        .await
        .map_err(|err| match err {
            DbSessionGetByKeyErr::NotFound(_) => AuthErr::Unauthorized(ERR_MSG_SESSION.to_string()),
            _ => AuthErr::InternalServer,
        })?;

    Ok((token, session.user))
}

pub fn auth_token_get(headers: &HeaderMap, header_name: header::HeaderName) -> Option<String> {
    headers
        .get(header_name)
        .inspect(|v| trace!("extract auth value raw {v:?}"))
        .and_then(|v| v.to_str().ok().and_then(|v| extract_auth_token_plain(v)))
        .inspect(|v| trace!("extract auth value cut {v:?}"))
}

fn extract_auth_token_plain(input: impl AsRef<str>) -> Option<String> {
    let input = input.as_ref();
    let input_len = input.len();

    let mut start = 0;
    let mut end = 0_usize;
    let mut stage = 0_usize;
    for (i, c) in input.chars().map(|v| v).enumerate() {
        if (c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') {
            if stage == 0 {
                stage = 1;
                start = i;
            }
            end = i;
            trace!("0 {c} cursor {start} end {end}");
            continue;
        }

        if stage == 1 && end.saturating_sub(start) == 19 && end < input_len {
            return Some(input[start..=end].to_string());
        }

        stage = 0;

        trace!("3 {c} cursor {start} end {end}");
    }

    if end.saturating_sub(start) == 19 && end < input_len {
        Some(input[start..=end].to_string())
    } else {
        None
    }
}
