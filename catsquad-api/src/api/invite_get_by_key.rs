use axum::{
    Form, Json,
    extract::{RawPathParams, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbInvite, DbInviteAddErr, DbInviteGetByKeyErr, id_to_string};
use catsquad_log::prelude::*;

use crate::{state::AppState, validation::validate_email, web::link_absolute_reg_finish};

pub const INVITE_GET_BY_KEY_ENDPOINT: &'static str = "/api/invite/{invite_key}";

pub fn link_relative_invite_get_by_key(invite_key: impl AsRef<str>) -> String {
    format!("/api/invite/{}", invite_key.as_ref())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteGetByKeyRes {
    pub email: String,
    pub expires: u128,
}

impl From<DbInvite> for InviteGetByKeyRes {
    fn from(value: DbInvite) -> Self {
        Self {
            email: value.email,
            expires: value.expires,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteGetByKeyReq {
    pub invite_key: String,
}
pub const INVITE_GET_BY_KEY_REQ_FIELD_INVITE_KEY: &'static str = "invite_key";

impl TryFrom<RawPathParams> for InviteGetByKeyReq {
    type Error = InviteGetByKeyErr;

    fn try_from(value: RawPathParams) -> Result<Self, Self::Error> {
        value
            .iter()
            .find(|(name, _)| *name == INVITE_GET_BY_KEY_REQ_FIELD_INVITE_KEY)
            .ok_or(InviteGetByKeyErr::BadRequest(
                "missing invite_key param".to_string(),
            ))
            .map(|(_, value)| InviteGetByKeyReq {
                invite_key: value.to_string(),
            })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum InviteGetByKeyErr {
    #[error("invite not found")]
    InviteNotFound,

    #[error("invite already used")]
    InviteAlreadyUsed,

    #[error("invite expired")]
    InviteExpired,

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("internal server err")]
    InternalServerErr,
}

impl From<DbInviteGetByKeyErr> for InviteGetByKeyErr {
    fn from(value: DbInviteGetByKeyErr) -> Self {
        match value {
            DbInviteGetByKeyErr::InviteNotFound => Self::InviteNotFound,
            DbInviteGetByKeyErr::InviteExpired => Self::InviteAlreadyUsed,
            DbInviteGetByKeyErr::InviteAlreadyUsed => Self::InviteExpired,
            DbInviteGetByKeyErr::Db(_) => Self::InternalServerErr,
        }
    }
}

pub fn status_code(result: &Result<InviteGetByKeyRes, InviteGetByKeyErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(InviteGetByKeyErr::InviteNotFound) => StatusCode::BAD_REQUEST,
        Err(InviteGetByKeyErr::InviteExpired) => StatusCode::BAD_REQUEST,
        Err(InviteGetByKeyErr::InviteAlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(InviteGetByKeyErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(InviteGetByKeyErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn invite_get_by_key(
    State(app): State<AppState>,
    params: axum::extract::RawPathParams,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<InviteGetByKeyRes, InviteGetByKeyErr> {
        let req = InviteGetByKeyReq::try_from(params)?;

        let invite = app.db.invite_get_by_key(time, req.invite_key).await?;

        Ok(InviteGetByKeyRes::from(invite))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod invite_add_test_utils {
    use crate::{
        TestServer,
        api::invite_get_by_key::{
            INVITE_GET_BY_KEY_ENDPOINT, InviteGetByKeyErr, InviteGetByKeyRes,
            link_relative_invite_get_by_key,
        },
    };

    impl TestServer {
        pub async fn invite_get_by_key(
            &self,
            invite_key: impl AsRef<str>,
        ) -> Result<InviteGetByKeyRes, InviteGetByKeyErr> {
            let link = link_relative_invite_get_by_key(invite_key);
            self.get::<Result<InviteGetByKeyRes, InviteGetByKeyErr>>(link)
                .await
        }
    }
}

#[tokio::test]
async fn test_invite_get_by_key() {
    init_log();
    let server = crate::TestServer::new().await;

    server.invite_add("prime@heyadora.com").await.unwrap();
    let invite_key = id_to_string(
        server.state.db.invite_get_all().await.unwrap()[0]
            .id
            .clone(),
    );

    let invite = server.invite_get_by_key(invite_key).await.unwrap();
    assert_eq!(invite.email, "prime@heyadora.com");

    let result = server.invite_get_by_key("invalid").await;
    assert_eq!(result, Err(InviteGetByKeyErr::InviteNotFound));
}
