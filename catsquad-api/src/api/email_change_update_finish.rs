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

fn status_code(result: &Result<(), EmailChangeUpdateFinishErr>) -> StatusCode {
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
    let time = app.get_time_micro();
    let inner = async || -> Result<(), EmailChangeUpdateFinishErr> {
        let user_username = db_user.username.clone();
        let email_change_id = req.email_change_id.clone();

        app.db
            .email_change_update_finish(time, user_username, email_change_id)
            .await
            .map_err(from_db_email_change_finish_err)?;

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
        pub async fn email_change_update_finish(
            &self,
            email_change_id: i64,
            session_token: Uuid,
        ) -> Result<(), cs::EmailChangeUpdateFinishErr> {
            self.client
                .email_change_update_finish(email_change_id)
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
async fn test_api_email_change_update_finish() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new(0, "test_api_email_change_update_finish").await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "a1234567890111GG11$")
        .await;

    let (user3, session_key3) = server
        .user_add_full("hey3", "hey3@heyadora.com", "a1234567890111GG11$")
        .await;

    server.state.set_time(0);
    server.state.set_email_change_expiration(10).await;

    // let finish = async |key: String, session: String| {
    //     server
    //         .client
    //         .email_change_update_finish(key)
    //         .header_add(header::COOKIE, create_auth_cookie_str(session))
    //         .send()
    //         .await
    //         .into_json()
    //         .await
    // };

    {
        let email_change = server.email_change_add(session_key).await.unwrap();

        let current_token = server
            .email_change_get_current_token(0, &user1, email_change.id)
            .await;

        server
            .email_change_update_current_confirm(
                email_change.id,
                current_token.clone(),
                session_key,
            )
            .await
            .unwrap();

        let result = server
            .email_change_update_finish(email_change.id, session_key.clone())
            .await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::NewEmailNotConfirmed)
        ));

        let email_change = server
            .email_change_update_new_add(email_change.id, "hey2@heyadora.com", session_key)
            .await
            .unwrap();

        let new_token = server
            .email_change_get_new_token(0, &user1, email_change.id)
            .await;

        let result = server
            .email_change_update_finish(email_change.id, session_key.clone())
            .await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::NewEmailNotConfirmed)
        ));

        server
            .email_change_update_new_confirm(email_change.id, new_token, session_key)
            .await
            .unwrap();

        let result = server.email_change_update_finish(0, session_key).await;
        assert!(matches!(result, Err(EmailChangeUpdateFinishErr::NotFound)));

        let result = server
            .email_change_update_finish(email_change.id, session_key3)
            .await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::Unauthorized(_))
        ));

        server.state.set_time(11);

        let result = server
            .email_change_update_finish(email_change.id, session_key)
            .await;
        assert!(matches!(result, Err(EmailChangeUpdateFinishErr::Expired)));
        server.state.set_time(0);

        server
            .email_change_update_finish(email_change.id, session_key)
            .await
            .unwrap();

        let result = server
            .email_change_update_finish(email_change.id, session_key)
            .await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::AlreadyUsed)
        ));
    }
    {
        let email_change = server.email_change_add(session_key).await.unwrap();

        let current_token = server
            .email_change_get_current_token(0, &user1, email_change.id)
            .await;

        server
            .email_change_update_current_confirm(
                email_change.id,
                current_token.clone(),
                session_key,
            )
            .await
            .unwrap();

        let email_change = server
            .email_change_update_new_add(email_change.id, "hey4@heyadora.com", session_key)
            .await
            .unwrap();

        let new_token = server
            .email_change_get_new_token(0, &user1, email_change.id)
            .await;

        server
            .email_change_update_new_confirm(email_change.id, new_token, session_key)
            .await
            .unwrap();

        let (_user4, _session_key4) = server
            .user_add_full("hey4", "hey4@heyadora.com", "a1234567890111GG11$")
            .await;

        let result = server
            .email_change_update_finish(email_change.id, session_key)
            .await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateFinishErr::EmailIsTaken)
        ));
    }
}
