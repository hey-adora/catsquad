use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeGetByIdErr, DbEmailChangeUpdateCancelErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{EmailChangeRes, EmailChangeUpdateCancelErr, EmailChangeUpdateCancelReq};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn from_db_email_change_update_cancel_err(
    value: DbEmailChangeUpdateCancelErr,
) -> EmailChangeUpdateCancelErr {
    match value {
        DbEmailChangeUpdateCancelErr::NotFound => EmailChangeUpdateCancelErr::NotFound,
        DbEmailChangeUpdateCancelErr::Unauthorized => {
            EmailChangeUpdateCancelErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeUpdateCancelErr::AlreadyUsed => EmailChangeUpdateCancelErr::AlreadyUsed,
        DbEmailChangeUpdateCancelErr::Expired => EmailChangeUpdateCancelErr::Expired,
        DbEmailChangeUpdateCancelErr::Db(_) => EmailChangeUpdateCancelErr::InternalServer,
    }
}

pub fn status_code(result: &Result<(), EmailChangeUpdateCancelErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeUpdateCancelErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCancelErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateCancelErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCancelErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCancelErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn email_change_update_cancel(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateCancelReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<(), EmailChangeUpdateCancelErr> {
        let user_username = db_user.username.clone();
        let email_change_key = req.email_change_id.clone();

        app.db
            .email_change_update_cancel(time, user_username, email_change_key)
            .await
            .map_err(from_db_email_change_update_cancel_err)?;

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
        pub async fn email_change_update_cancel(
            &self,
            email_change_id: i64,
            session_token: Uuid,
        ) -> Result<(), cs::EmailChangeUpdateCancelErr> {
            self.client
                .email_change_update_cancel(email_change_id)
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

#[tokio::test]
async fn test_api_email_change_update_cancel() {
    use catsquad_shared::EmailChangeUpdateFinishErr;

    init_log();
    let server = crate::TestServer::new(0, "test_api_email_chang_updatee_cancel").await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "a1234567890111GG11$")
        .await;

    let (user2, session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "a1234567890111GG11$")
        .await;

    server.state.set_time(0);
    server.state.set_email_change_expiration(10).await;
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
            .email_change_update_new_add(email_change.id, "hey3@heyadora.com", session_key)
            .await
            .unwrap();

        let new_token = server
            .email_change_get_new_token(0, &user1, email_change.id)
            .await;

        server
            .email_change_update_new_confirm(email_change.id, new_token, session_key)
            .await
            .unwrap();

        let result = server.email_change_update_cancel(0, session_key).await;
        assert!(matches!(result, Err(EmailChangeUpdateCancelErr::NotFound)));

        let result = server
            .email_change_update_cancel(email_change.id, session_key2)
            .await;
        assert!(matches!(
            result,
            Err(EmailChangeUpdateCancelErr::Unauthorized(_))
        ));

        server.state.set_time(11);
        let result = server
            .email_change_update_cancel(email_change.id, session_key)
            .await;
        assert!(matches!(result, Err(EmailChangeUpdateCancelErr::Expired)));
        server.state.set_time(0);

        server
            .email_change_update_cancel(email_change.id, session_key)
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
        assert!(!email_change.completed);
        server
            .email_change_update_cancel(email_change.id, session_key)
            .await
            .unwrap();
        let result = server
            .state
            .db
            .email_change_get_by_id(0, user1.username, email_change.id)
            .await;
        assert!(matches!(result, Err(DbEmailChangeGetByIdErr::AlreadyUsed)));
    }
}
