use axum::{extract::FromRequestParts, http::header};
use catsquad_log::prelude::*;
use catsquad_shared::Uuid;
use std::{ascii::AsciiExt, char};

pub fn get_time_ns() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    time.as_nanos()
}

pub fn get_time_micro() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    time.as_micros()
}

pub fn rng_str(len: usize) -> String {
    use rand::distr::SampleString;
    rand::distr::Alphanumeric.sample_string(&mut rand::rng(), len)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserAgent(pub String);

impl<S> FromRequestParts<S> for UserAgent
where
    S: Send + Sync,
{
    type Rejection = String;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        trace!("ALL HEADERS HERE {headers:#?}");
        let user_agent = headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().map(|v| v.to_string()).ok())
            .unwrap_or_default();

        Ok(Self(user_agent))
    }
}
