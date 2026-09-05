use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeUpdateNewAddErr, DbEmailSentReason, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    EmailChangeRes, EmailChangeUpdateNewAddErr, EmailChangeUpdateNewAddReq, Uuid,
    link_absolute_settings_email_change_new_confirm, uuid_to_str, validate_email,
};
use std::fmt::Display;
use url::Url;

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
        Err(EmailChangeUpdateNewAddErr::EmailIsInvalid(_)) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn send_email_email_change_update_new_add(
    address: Url,
    email_change_key: impl Display,
    token: Uuid,
) -> String {
    let token_str = uuid_to_str(token);
    let link =
        link_absolute_settings_email_change_new_confirm(address, email_change_key, token_str)
            .unwrap()
            .to_string();
    // let link = "placeholder change".to_string();
    // debug!("EMAIL SENT {link}");
    link
}

pub async fn email_change_update_new_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateNewAddReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<EmailChangeRes, EmailChangeUpdateNewAddErr> {
        let user_username = db_user.username.clone();
        let user_email = db_user.email.clone();
        let email_change_id = req.email_change_id;

        let new_email = req.new_email.trim().to_lowercase();
        validate_email(&new_email)
            .map_err(|err| EmailChangeUpdateNewAddErr::EmailIsInvalid(err))?;

        app.db
            .email_change_update_new_add(time, user_username.clone(), email_change_id, new_email)
            .await
            .map_err(from_db_email_change_update_new_add_err)?;

        // TODO add better error and maybe combine queries
        let email_change = app
            .db
            .email_change_get_by_key(time, user_username, email_change_id)
            .await
            .map_err(|_| EmailChangeUpdateNewAddErr::InternalServer)?;
        let new_token = email_change.new_token;
        // let email_change
        // let key = id_to_string(email_change.id.clone());

        let address = app.get_address().await;
        let email_body =
            send_email_email_change_update_new_add(address, email_change.id, new_token);
        let _ = app
            .db
            .email_sent_add(
                time,
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
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn email_change_update_new_add(
            &self,
            email_change_id: i64,
            new_email: impl Into<String>,
            session_token: Uuid,
        ) -> Result<cs::EmailChangeRes, cs::EmailChangeUpdateNewAddErr> {
            self.client
                .email_change_update_new_add(email_change_id, new_email)
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
async fn test_api_email_change_update_new_add() {
    use axum::http::header;

    use crate::auth::create_auth_cookie_str;

    init_log();
    let server = crate::TestServer::new(0, "test_api_email_change_update_new_add").await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "12a34567890111GG11$")
        .await;
    let (user5, session_key5) = server
        .user_add_full("hey5", "hey5@heyadora.com", "12a34567890111GG11$")
        .await;
    server.state.set_email_change_expiration(10).await;

    let email_change = server.email_change_add(session_key).await.unwrap();

    let current_token = server
        .email_change_get_current_token(0, &user1, email_change.id)
        .await;

    let add_new = async |key: i64, email: &str, session: Uuid| {
        server
            .email_change_update_new_add(key, email, session)
            .await
    };

    let result = add_new(email_change.id, "hey2@heyadora.com", session_key).await;

    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::NotConfirmed)
    ));

    server
        .email_change_update_current_confirm(email_change.id, current_token, session_key)
        .await
        .unwrap();

    let result = add_new(0, "hey2@heyadora.com", session_key).await;
    assert!(matches!(result, Err(EmailChangeUpdateNewAddErr::NotFound)));

    let result = add_new(0, "hey2@heyadora.com", 0_u128.to_be_bytes()).await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::Unauthorized(_))
    ));

    server.state.set_time(11);
    let result = add_new(email_change.id, "hey2@heyadora.com", session_key.clone()).await;
    assert!(matches!(result, Err(EmailChangeUpdateNewAddErr::Expired)));
    server.state.set_time(0);

    let result = add_new(email_change.id, "hey5@heyadora.com", session_key.clone()).await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::EmailIsTaken(_))
    ));

    let result = add_new(email_change.id, "hey2", session_key.clone()).await;
    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::EmailIsInvalid(_))
    ));

    let result = add_new(email_change.id, "hey2@heyadora.com", session_key.clone())
        .await
        .unwrap();

    let result = add_new(email_change.id, "hey2@heyadora.com", session_key.clone()).await;

    assert!(matches!(
        result,
        Err(EmailChangeUpdateNewAddErr::AlreadyUsed)
    ));
}
