use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeUpdateFinishErr, DbUser};
use catsquad_shared::{EmailChangeRes, EmailChangeUpdateFinishErr, EmailChangeUpdateFinishReq};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn from_db_email_change_finish_err(
    value: DbEmailChangeUpdateFinishErr,
) -> EmailChangeUpdateFinishErr {
    match value {
        DbEmailChangeUpdateFinishErr::NotFound => EmailChangeUpdateFinishErr::NotFound,
        DbEmailChangeUpdateFinishErr::Unauthorized => {
            EmailChangeUpdateFinishErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeUpdateFinishErr::AlreadyUsed => EmailChangeUpdateFinishErr::AlreadyUsed,
        DbEmailChangeUpdateFinishErr::Expired => EmailChangeUpdateFinishErr::Expired,
        DbEmailChangeUpdateFinishErr::NewEmailNotConfirmed => {
            EmailChangeUpdateFinishErr::NewEmailNotConfirmed
        }
        DbEmailChangeUpdateFinishErr::EmailIsTaken => EmailChangeUpdateFinishErr::EmailIsTaken,
        DbEmailChangeUpdateFinishErr::Db(_) => EmailChangeUpdateFinishErr::InternalServer,
    }
}

fn status_code(result: &Result<EmailChangeRes, EmailChangeUpdateFinishErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeUpdateFinishErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateFinishErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateFinishErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateFinishErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateFinishErr::NewEmailNotConfirmed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateFinishErr::EmailIsTaken) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateFinishErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn email_change_update_finish(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateFinishReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<EmailChangeRes, EmailChangeUpdateFinishErr> {
        let user_id = db_user.id.clone();
        let email_change_key = req.email_change_key.clone();

        let email_change = app
            .db
            .email_change_update_finish(time, user_id, email_change_key)
            .await
            .map_err(from_db_email_change_finish_err)?;

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
        pub async fn email_change_update_finish(
            &self,
            email_change_key: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::EmailChangeRes, cs::EmailChangeUpdateFinishErr> {
            self.client
                .email_change_update_finish(email_change_key)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_res()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_update_finish() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new().await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "a1234567890111GG11$")
        .await;

    let (user3, session_key3) = server
        .user_add_full("hey3", "hey3@heyadora.com", "a1234567890111GG11$")
        .await;

    server.state.set_time(0).await;
    server.state.set_email_change_expiration(10).await;

    let finish = async |key: String, session: String| {
        server
            .client
            .email_change_update_finish(key)
            .header_add(header::COOKIE, create_auth_cookie_str(session))
            .send()
            .await
            .into_res()
            .await
    };

    {
        let email_change = server
            .client
            .email_change_add()
            .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
            .send()
            .await
            .into_res()
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

        let email_change = server
            .client
            .email_change_update_current_confirm(email_change.key.clone(), current_token.clone())
            .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
            .send()
            .await
            .into_res()
            .await
            .unwrap();

        let result = finish(email_change.key.clone(), session_key.clone()).await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::NewEmailNotConfirmed)
        ));

        let email_change = server
            .client
            .email_change_update_new_add(email_change.key.clone(), "hey2@heyadora.com")
            .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
            .send()
            .await
            .into_res()
            .await
            .unwrap();

        let new_token = server
            .state
            .db
            .email_change_get_by_key(email_change.key.clone())
            .await
            .unwrap()
            .new
            .unwrap()
            .token;

        let result = finish(email_change.key.clone(), session_key.clone()).await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::NewEmailNotConfirmed)
        ));

        let email_change = server
            .client
            .email_change_update_new_confirm(email_change.key.clone(), new_token)
            .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
            .send()
            .await
            .into_res()
            .await
            .unwrap();

        let result = finish("invalid".to_string(), session_key.clone()).await;
        assert!(matches!(result, Err(EmailChangeUpdateFinishErr::NotFound)));

        let result = finish(email_change.key.clone(), session_key3.clone()).await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::Unauthorized(_))
        ));

        server.state.set_time(11).await;

        let result = finish(email_change.key.clone(), session_key.clone()).await;
        assert!(matches!(result, Err(EmailChangeUpdateFinishErr::Expired)));
        server.state.set_time(0).await;

        let email_change = finish(email_change.key.clone(), session_key.clone())
            .await
            .unwrap();

        let result = finish(email_change.key.clone(), session_key.clone()).await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::AlreadyUsed)
        ));
    }
    {
        let email_change = server
            .client
            .email_change_add()
            .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
            .send()
            .await
            .into_res()
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

        let email_change = server
            .client
            .email_change_update_current_confirm(email_change.key.clone(), current_token.clone())
            .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
            .send()
            .await
            .into_res()
            .await
            .unwrap();

        let email_change = server
            .client
            .email_change_update_new_add(email_change.key.clone(), "hey4@heyadora.com")
            .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
            .send()
            .await
            .into_res()
            .await
            .unwrap();

        let new_token = server
            .state
            .db
            .email_change_get_by_key(email_change.key.clone())
            .await
            .unwrap()
            .new
            .unwrap()
            .token;

        let email_change = server
            .client
            .email_change_update_new_confirm(email_change.key.clone(), new_token)
            .header_add(header::COOKIE, create_auth_cookie_str(session_key.clone()))
            .send()
            .await
            .into_res()
            .await
            .unwrap();

        let (_user4, _session_key4) = server
            .user_add_full("hey4", "hey4@heyadora.com", "a1234567890111GG11$")
            .await;

        let result = finish(email_change.key.clone(), session_key.clone()).await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::EmailIsTaken)
        ));
    }
}
