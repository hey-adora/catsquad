use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeUpdateNewAddErr, DbEmailSentReason, DbUser, id_to_string};
use catsquad_log::prelude::*;
use catsquad_shared::{EmailChangeRes, EmailChangeUpdateNewAddErr, EmailChangeUpdateNewAddReq};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn from_db_email_change_update_new_add_err(
    value: DbEmailChangeUpdateNewAddErr,
) -> EmailChangeUpdateNewAddErr {
    match value {
        DbEmailChangeUpdateNewAddErr::NotFound => EmailChangeUpdateNewAddErr::NotFound,
        DbEmailChangeUpdateNewAddErr::Unauthorized => {
            EmailChangeUpdateNewAddErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeUpdateNewAddErr::AlreadyUsed => EmailChangeUpdateNewAddErr::AlreadyUsed,
        DbEmailChangeUpdateNewAddErr::Expired => EmailChangeUpdateNewAddErr::Expired,
        DbEmailChangeUpdateNewAddErr::NotConfirmed => EmailChangeUpdateNewAddErr::NotConfirmed,
        DbEmailChangeUpdateNewAddErr::EmailIsTaken(v) => {
            EmailChangeUpdateNewAddErr::EmailIsTaken(v)
        }
        DbEmailChangeUpdateNewAddErr::Db(_) => EmailChangeUpdateNewAddErr::InternalServer,
    }
}

fn status_code(result: &Result<EmailChangeRes, EmailChangeUpdateNewAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeUpdateNewAddErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateNewAddErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewAddErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewAddErr::NotConfirmed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewAddErr::EmailIsTaken(_)) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_email_change_update_new_add(
    address: impl Into<String>,
    email_change_key: impl Into<String>,
    token: impl Into<String>,
) -> String {
    // let link = link_absolute_reg_finish(address, token);
    let link = "placeholder change".to_string();
    debug!("EMAIL SENT {link}");
    link
}

pub async fn email_change_update_new_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateNewAddReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<EmailChangeRes, EmailChangeUpdateNewAddErr> {
        let user_id = db_user.id.clone();
        let user_email = db_user.email.clone();
        let email_change_key = req.email_change_key;
        let new_email = req.new_email;

        let email_change = app
            .db
            .email_change_update_new_add(time, user_id, email_change_key, new_email)
            .await
            .map_err(from_db_email_change_update_new_add_err)?;
        let token = email_change.current.token.clone();
        let key = id_to_string(email_change.id.clone());

        let address = app.get_address().await;
        let email_body = send_email_email_change_update_new_add(address, key, token);
        let _ = app
            .db
            .email_sent_add(
                0,
                DbEmailSentReason::UserEmailChangeAddNew,
                user_email,
                email_body,
            )
            .await;

        Ok(from_db_email_change(email_change))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared as cs;

    impl TestServer {
        pub async fn email_change_update_new_add(
            &self,
            email_change_key: impl Into<String>,
            new_email: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::EmailChangeRes, cs::EmailChangeUpdateNewAddErr> {
            self.client
                .email_change_update_new_add(email_change_key, new_email)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_update_new_add() {
    use axum::http::header;

    use crate::auth::create_auth_cookie_str;

    init_log();
    let server = crate::TestServer::new().await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "12a34567890111GG11$")
        .await;
    let (user5, session_key5) = server
        .user_add_full("hey5", "hey5@heyadora.com", "12a34567890111GG11$")
        .await;
    server.state.set_email_change_expiration(10).await;

    let email_change = server
        .client
        .email_change_add()
        .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    let current_token = server
        .state
        .db
        .email_change_get_by_key(email_change.key.clone())
        .await
        .unwrap()
        .current
        .token;

    let add_new = async |key: String, email: &str, session: String| {
        server
            .client
            .email_change_update_new_add(key, email)
            .header_add(header::COOKIE, create_auth_cookie_str(session))
            .send()
            .await
            .into_json()
            .await
    };

    let result = add_new(
        email_change.key.to_string(),
        "hey2@heyadora.com",
        session_key.clone(),
    )
    .await;

    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::NotConfirmed)
    ));

    let email_change = server
        .client
        .email_change_update_current_confirm(email_change.key.clone(), current_token.clone())
        .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    let result = add_new(
        "invalid".to_string(),
        "hey2@heyadora.com",
        session_key.clone(),
    )
    .await;
    assert!(matches!(result, Err(EmailChangeUpdateNewAddErr::NotFound)));

    let result = add_new(
        "invalid".to_string(),
        "hey2@heyadora.com",
        "invalid".to_string(),
    )
    .await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::Unauthorized(_))
    ));

    server.state.set_time(11).await;
    let result = add_new(
        email_change.key.clone(),
        "hey2@heyadora.com",
        session_key.clone(),
    )
    .await;
    assert!(matches!(result, Err(EmailChangeUpdateNewAddErr::Expired)));
    server.state.set_time(0).await;

    let result = add_new(
        email_change.key.clone(),
        "hey5@heyadora.com",
        session_key.clone(),
    )
    .await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::EmailIsTaken(_))
    ));

    let result = add_new(
        email_change.key.clone(),
        "hey2@heyadora.com",
        session_key.clone(),
    )
    .await
    .unwrap();

    let result = add_new(
        email_change.key.clone(),
        "hey2@heyadora.com",
        session_key.clone(),
    )
    .await;

    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::AlreadyUsed)
    ));
}
