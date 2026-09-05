use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeConfirmUpdateCurrentErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    EmailChangeRes, EmailChangeUpdateCurrentConfirmErr, EmailChangeUpdateCurrentConfirmReq,
};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn from_db_email_change_confirm_current_err(
    value: DbEmailChangeConfirmUpdateCurrentErr,
) -> EmailChangeUpdateCurrentConfirmErr {
    match value {
        DbEmailChangeConfirmUpdateCurrentErr::NotFound => {
            EmailChangeUpdateCurrentConfirmErr::NotFound
        }
        DbEmailChangeConfirmUpdateCurrentErr::Unauthorized => {
            EmailChangeUpdateCurrentConfirmErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeConfirmUpdateCurrentErr::AlreadyUsed => {
            EmailChangeUpdateCurrentConfirmErr::AlreadyUsed
        }
        DbEmailChangeConfirmUpdateCurrentErr::Expired => {
            EmailChangeUpdateCurrentConfirmErr::Expired
        }
        DbEmailChangeConfirmUpdateCurrentErr::InvalidToken => {
            EmailChangeUpdateCurrentConfirmErr::InvalidToken
        }
        DbEmailChangeConfirmUpdateCurrentErr::Db(_) => {
            EmailChangeUpdateCurrentConfirmErr::InternalServer
        }
    }
}

pub fn status_code(result: &Result<(), EmailChangeUpdateCurrentConfirmErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeUpdateCurrentConfirmErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCurrentConfirmErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateCurrentConfirmErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCurrentConfirmErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCurrentConfirmErr::InvalidToken) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateCurrentConfirmErr::InternalServer) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn email_change_update_current_confirm(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateCurrentConfirmReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<(), EmailChangeUpdateCurrentConfirmErr> {
        let user_username = db_user.username.clone();
        let email_change_key = req.email_change_id.clone();
        let email_change_token = req.token.clone();

        app.db
            .email_change_update_current_confirm(
                time,
                user_username,
                email_change_key,
                email_change_token,
            )
            .await
            .map_err(from_db_email_change_confirm_current_err)?;

        Ok(())
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn email_change_update_current_confirm(
            &self,
            email_change_id: i64,
            token: Uuid,
            session_token: Uuid,
        ) -> Result<(), cs::EmailChangeUpdateCurrentConfirmErr> {
            self.client
                .email_change_update_current_confirm(email_change_id, token)
                .header_add(
                    header::COOKIE,
                    create_auth_cookie_str(uuid_to_str(session_token)),
                )
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_email_change_update_current_confirm() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;
    init_log();
    let server = crate::TestServer::new(0, "test_api_email_change_update_current_confirm").await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "1234567890111GG2f11$")
        .await;

    let (user2, session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "1234567890111GGg11$")
        .await;

    {
        server.state.set_email_change_expiration(10).await;

        // let email_change = server.email_change_add(&session_key).await.unwrap();
        let email_change = server.email_change_add(session_key).await.unwrap();

        let current_token = server
            .email_change_get_current_token(0, &user1, email_change.id)
            .await;

        let result = server
            .email_change_update_current_confirm(0, 0_u128.to_be_bytes(), session_key)
            .await;

        assert!(matches!(
            result,
            Err(EmailChangeUpdateCurrentConfirmErr::NotFound)
        ));

        let result = server
            .email_change_update_current_confirm(email_change.id, 0_u128.to_be_bytes(), session_key)
            .await;

        assert!(matches!(
            result,
            Err(EmailChangeUpdateCurrentConfirmErr::InvalidToken)
        ));

        let result = server
            .email_change_update_current_confirm(
                email_change.id,
                0_u128.to_be_bytes(),
                session_key2.clone(),
            )
            .await;

        assert!(matches!(
            result,
            Err(EmailChangeUpdateCurrentConfirmErr::Unauthorized(_))
        ));

        server.state.set_time(11);

        let result = server
            .email_change_update_current_confirm(
                email_change.id,
                current_token.clone(),
                session_key,
            )
            .await;

        assert!(matches!(
            result,
            Err(EmailChangeUpdateCurrentConfirmErr::Expired)
        ));
        server.state.set_time(0);

        server
            .email_change_update_current_confirm(email_change.id, current_token, session_key)
            .await
            .unwrap();

        let result = server
            .email_change_update_current_confirm(
                email_change.id,
                current_token.clone(),
                session_key,
            )
            .await;

        assert!(matches!(
            result,
            Err(EmailChangeUpdateCurrentConfirmErr::AlreadyUsed)
        ));
    }
}
