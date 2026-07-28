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
    InviteGetByKeyRes,
};

use crate::state::AppState;

fn from_db_invite(value: DbInvite) -> InviteGetByKeyRes {
    InviteGetByKeyRes {
        email: value.email,
        expires: value.expires,
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
    let time = app.get_time().await;
    let inner = async || -> Result<InviteGetByKeyRes, InviteGetByKeyErr> {
        let req = params_req(params)?;

        let invite = app
            .db
            .invite_get_by_key(time, req.invite_key)
            .await
            .map_err(from_db_invite_get_by_key_err)?;

        Ok(from_db_invite(invite))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

// #[cfg(test)]
// mod test_utils {
//     use catsquad_shared::link_relative_invite_get_by_key;

//     use crate::{
//         TestServer,
//         api::invite_get_by_key::{InviteGetByKeyErr, InviteGetByKeyRes},
//     };

//     impl TestServer {
//         pub async fn invite_get_by_key(
//             &self,
//             invite_key: impl AsRef<str>,
//         ) -> Result<InviteGetByKeyRes, InviteGetByKeyErr> {
//             let link = link_relative_invite_get_by_key(invite_key);
//             self.get::<Result<InviteGetByKeyRes, InviteGetByKeyErr>>(link)
//                 .await
//         }
//     }
// }

#[tokio::test]
async fn test_invite_get_by_key() {
    use catsquad_db::id_to_string;

    init_log();
    let server = crate::TestServer::new().await;

    server
        .client
        .invite_add("prime@heyadora.com")
        .await
        .send()
        .await
        .into_res()
        .await
        .unwrap();
    let invite_key = id_to_string(
        server.state.db.invite_get_all().await.unwrap()[0]
            .id
            .clone(),
    );

    let invite = server
        .client
        .invite_get_by_key(invite_key)
        .await
        .send()
        .await
        .into_res()
        .await
        .unwrap();
    assert_eq!(invite.email, "prime@heyadora.com");

    let result = server
        .client
        .invite_get_by_key("invalid")
        .await
        .send()
        .await
        .into_res()
        .await;

    assert_eq!(result, Err(InviteGetByKeyErr::InviteNotFound));
}
