use axum::{Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{
    DbEmailSentAddErr, DbInvite, DbInviteAddErr, DbUserUpdatePasswordByIdErr, id_to_string,
};
use catsquad_log::prelude::*;

use crate::{state::AppState, validation::validate_email, web::link_absolute_reg_finish};

pub const INVITE_ADD_ENDPOINT: &'static str = "/api/invite";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteAddRes {
    pub expires: u128,
}

impl From<DbInvite> for InviteAddRes {
    fn from(value: DbInvite) -> Self {
        Self {
            expires: value.expires,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InviteAddReq {
    pub email: String,
}
pub const INVITE_ADD_REQ_FIELD_EMAIL: &'static str = "email";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error)]
pub enum InviteAddErr {
    #[error("email is invalid")]
    InvalidEmail(String),

    #[error("bad request {0}")]
    BadRequest(String),

    #[error("internal server err")]
    InternalServerErr,
}

pub fn status_code(result: &Result<InviteAddRes, InviteAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(InviteAddErr::InvalidEmail(_)) => StatusCode::BAD_REQUEST,
        Err(InviteAddErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(InviteAddErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn send_email(address: impl AsRef<str>, token: impl AsRef<str>) -> String {
    let link = link_absolute_reg_finish(address, token);
    debug!("EMAIL SENT {link}");
    link
}

pub async fn invite_add(
    State(app): State<AppState>,
    Form(req): Form<InviteAddReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let invite_expiration = app.get_invite_expiration().await;
    let inner = async || -> Result<InviteAddRes, InviteAddErr> {
        let email = req.email.trim().to_lowercase();
        validate_email(&email).map_err(|err| InviteAddErr::InvalidEmail(err))?;

        let expires = time + invite_expiration;
        let result = app.db.invite_add(time, &email, expires).await;
        let result = match result {
            Ok(v) => v,
            Err(DbInviteAddErr::EmailIsTaken(_)) => return Ok(InviteAddRes { expires }),
            Err(DbInviteAddErr::Db(_)) => return Err(InviteAddErr::InternalServerErr),
        };

        let address = app.get_address().await;
        let email_body = send_email(address, &id_to_string(result.id));
        let _ = app
            .db
            .email_sent_add(
                0,
                catsquad_db::DbEmailSentReason::ConfirmInvite,
                email,
                email_body,
            )
            .await;

        Ok(InviteAddRes {
            expires: result.expires,
        })
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod invite_add_test_utils {
    use crate::{
        TestServer,
        api::{
            INVITE_ADD_ENDPOINT,
            invite_add::{INVITE_ADD_REQ_FIELD_EMAIL, InviteAddErr, InviteAddRes},
        },
    };

    impl TestServer {
        pub async fn invite_add(
            &self,
            email: impl AsRef<str>,
        ) -> Result<InviteAddRes, InviteAddErr> {
            let data = format!("{}={}", INVITE_ADD_REQ_FIELD_EMAIL, email.as_ref());
            self.post::<Result<InviteAddRes, InviteAddErr>>(INVITE_ADD_ENDPOINT, data)
                .await
        }
    }
}

#[tokio::test]
async fn test_invite_add() {
    init_log();
    let server = crate::TestServer::new().await;

    let result = server.invite_add("hello").await;
    assert!(matches!(result, Err(InviteAddErr::InvalidEmail(_))));

    let result = server.invite_add("prime@heyadora.com").await;
    assert!(result.is_ok());
}
