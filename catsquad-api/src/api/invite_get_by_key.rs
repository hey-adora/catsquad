use axum::{
    Json,
    extract::{RawPathParams, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbInvite, DbInviteGetByKeyErr};
use catsquad_log::prelude::*;
use catsquad_shared::{
    INVITE_GET_BY_KEY_REQ_FIELD_INVITE_KEY, InviteGetByKeyErr, InviteGetByKeyParams,
    InviteGetByKeyRes, str_to_uuid,
};

use crate::state::AppState;

fn from_db_invite(value: DbInvite) -> InviteGetByKeyRes {
    InviteGetByKeyRes {
        email: value.email,
        expires: value.expires_at,
    }
}

pub fn from_db_invite_get_by_key_err(value: DbInviteGetByKeyErr) -> InviteGetByKeyErr {
    match value {
        DbInviteGetByKeyErr::InviteNotFound => InviteGetByKeyErr::InviteNotFound,
        DbInviteGetByKeyErr::InviteExpired => InviteGetByKeyErr::InviteAlreadyUsed,
        DbInviteGetByKeyErr::InviteAlreadyUsed => InviteGetByKeyErr::InviteExpired,
        DbInviteGetByKeyErr::Db(_) => InviteGetByKeyErr::InternalServerErr,
    }
}

fn params_req(value: RawPathParams) -> Result<InviteGetByKeyParams, InviteGetByKeyErr> {
    value
        .iter()
        .find(|(name, _)| *name == INVITE_GET_BY_KEY_REQ_FIELD_INVITE_KEY)
        .ok_or(InviteGetByKeyErr::BadRequest(
            "missing invite_key param".to_string(),
        ))
        .map(|(_, value)| InviteGetByKeyParams {
            invite_key: value.to_string(),
        })
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
    let time = app.get_time_micro();
    let inner = async || -> Result<InviteGetByKeyRes, InviteGetByKeyErr> {
        let req = params_req(params)?;
        let invite_key = str_to_uuid(&req.invite_key);

        let invite = app
            .db
            .invite_get_by_key(time, invite_key)
            .await
            .map_err(from_db_invite_get_by_key_err)?;

        Ok(from_db_invite(invite))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use catsquad_shared::{self as cs, Uuid};

    use crate::TestServer;

    impl TestServer {
        pub async fn invite_get_by_key(
            &self,
            invite_key: Uuid,
        ) -> Result<cs::InviteGetByKeyRes, cs::InviteGetByKeyErr> {
            self.client
                .invite_get_by_key(invite_key)
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[tokio::test]
async fn test_invite_get_by_key() {
    init_log();
    let server = crate::TestServer::new().await;

    server.invite_add("prime@heyadora.com").await.unwrap();
    let invite_key = server.state.db.invite_get_all().await.unwrap()[0].token;

    let invite = server.invite_get_by_key(invite_key).await.unwrap();
    assert_eq!(invite.email, "prime@heyadora.com");

    let result = server.invite_get_by_key(0_u128.to_be_bytes()).await;

    assert_eq!(result, Err(InviteGetByKeyErr::InviteNotFound));
}
