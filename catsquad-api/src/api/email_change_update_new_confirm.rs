use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeUpdateNewConfirmErr, DbUser};
use catsquad_shared::{
    EmailChangeRes, EmailChangeUpdateNewConfirmErr, EmailChangeUpdateNewConfirmReq,
};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn from_db_email_change_update_new_confirm_err(
    value: DbEmailChangeUpdateNewConfirmErr,
) -> EmailChangeUpdateNewConfirmErr {
    match value {
        DbEmailChangeUpdateNewConfirmErr::NotFound => EmailChangeUpdateNewConfirmErr::NotFound,
        DbEmailChangeUpdateNewConfirmErr::Unauthorized => {
            EmailChangeUpdateNewConfirmErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeUpdateNewConfirmErr::AlreadyUsed => {
            EmailChangeUpdateNewConfirmErr::AlreadyUsed
        }
        DbEmailChangeUpdateNewConfirmErr::Expired => EmailChangeUpdateNewConfirmErr::Expired,
        DbEmailChangeUpdateNewConfirmErr::NewEmailNotSet => {
            EmailChangeUpdateNewConfirmErr::NewEmailNotSet
        }
        DbEmailChangeUpdateNewConfirmErr::InvalidToken => {
            EmailChangeUpdateNewConfirmErr::InvalidToken
        }
        DbEmailChangeUpdateNewConfirmErr::Db(_) => EmailChangeUpdateNewConfirmErr::InternalServer,
    }
}

fn status_code(result: &Result<(), EmailChangeUpdateNewConfirmErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeUpdateNewConfirmErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewConfirmErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateNewConfirmErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewConfirmErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewConfirmErr::NewEmailNotSet) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewConfirmErr::InvalidToken) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateNewConfirmErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn email_change_update_new_confirm(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateNewConfirmReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<(), EmailChangeUpdateNewConfirmErr> {
        let user_username = db_user.username.clone();
        let email_change_key = req.email_change_id.clone();
        let email_change_token = req.token.clone();

        let email_change = app
            .db
            .email_change_update_new_confirm(
                time,
                user_username,
                email_change_key,
                email_change_token,
            )
            .await
            .map_err(from_db_email_change_update_new_confirm_err)?;

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
        pub async fn email_change_update_new_confirm(
            &self,
            email_change_id: i64,
            token: Uuid,
            session_token: Uuid,
        ) -> Result<(), cs::EmailChangeUpdateNewConfirmErr> {
            self.client
                .email_change_update_new_confirm(email_change_id, token)
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
async fn test_email_change_update_new_confirm() {
    use axum::http::header;
    use catsquad_log::prelude::*;
    use catsquad_shared::Uuid;

    use crate::auth::create_auth_cookie_str;

    init_log();
    let server = crate::TestServer::new().await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "w1234567890111GG11$")
        .await;

    let (_user2, session_key2) = server
        .user_add_full("hey2", "hey3@heyadora.com", "w1234567890111GG11$")
        .await;

    server.state.set_time(0);
    server.state.set_email_change_expiration(10).await;

    let email_change = server.email_change_add(session_key).await.unwrap();

    let current_token = server
        .email_change_get_current_token(0, &user1, email_change.id)
        .await;

    let email_change = server
        .email_change_update_current_confirm(email_change.id, current_token.clone(), session_key)
        .await
        .unwrap();

    let new_confirm = async |key: i64, token: Uuid, session: Uuid| {
        server
            .email_change_update_new_confirm(key, token, session)
            .await
    };

    let result = new_confirm(email_change.id, 0_u128.to_be_bytes(), session_key.clone()).await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewConfirmErr::NewEmailNotSet)
    ));

    let email_change = server
        .email_change_update_new_add(email_change.id, "hey2@heyadora.com", session_key)
        .await
        .unwrap();

    let new_token = server
        .email_change_get_new_token(0, &user1, email_change.id)
        .await;

    let result = new_confirm(email_change.id, new_token, session_key2).await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewConfirmErr::Unauthorized(_))
    ));

    let result = new_confirm(0, new_token, session_key).await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewConfirmErr::NotFound)
    ));

    let result = new_confirm(email_change.id, 0_u128.to_be_bytes(), session_key.clone()).await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewConfirmErr::InvalidToken)
    ));

    server.state.set_time(11);

    let result = new_confirm(email_change.id, new_token, session_key).await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewConfirmErr::Expired)
    ));
    server.state.set_time(0);

    let result = new_confirm(email_change.id, current_token, session_key).await;

    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewConfirmErr::InvalidToken)
    ));

    new_confirm(email_change.id, new_token, session_key)
        .await
        .unwrap();

    let result = new_confirm(email_change.id, new_token, session_key).await;

    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewConfirmErr::AlreadyUsed)
    ));
}
