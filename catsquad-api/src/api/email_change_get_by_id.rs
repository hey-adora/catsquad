use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbEmailChangeGetByIdErr, DbUser};
use catsquad_shared::{EmailChangeGetByIdErr, EmailChangeGetByIdParams, EmailChangeRes};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn status_code(result: &Result<EmailChangeRes, EmailChangeGetByIdErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeGetByIdErr::NotFound) => StatusCode::NOT_FOUND,
        Err(EmailChangeGetByIdErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeGetByIdErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeGetByIdErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeGetByIdErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn from_db_email_change_gey_by_id_err(value: DbEmailChangeGetByIdErr) -> EmailChangeGetByIdErr {
    match value {
        DbEmailChangeGetByIdErr::Unauthorized => {
            EmailChangeGetByIdErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeGetByIdErr::Expired => EmailChangeGetByIdErr::Expired,
        DbEmailChangeGetByIdErr::AlreadyUsed => EmailChangeGetByIdErr::AlreadyUsed,
        DbEmailChangeGetByIdErr::EmailChangeNotFound => EmailChangeGetByIdErr::NotFound,
        DbEmailChangeGetByIdErr::Db(_) => EmailChangeGetByIdErr::InternalServer,
    }
}

pub async fn email_change_get_by_id(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Path(req): Path<EmailChangeGetByIdParams>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<EmailChangeRes, EmailChangeGetByIdErr> {
        let email_change_id = req.email_change_id;
        let user_username = db_user.username.clone();

        let result = app
            .db
            .email_change_get_by_id(time, user_username, email_change_id)
            .await
            .map_err(from_db_email_change_gey_by_id_err)?;

        Ok(from_db_email_change(result))
    };

    let result = inner().await;
    let status_code = status_code(&result);
    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn email_change_get_by_id(
            &self,
            email_change_id: i64,
            session_token: Uuid,
        ) -> Result<cs::EmailChangeRes, cs::EmailChangeGetByIdErr> {
            self.client
                .email_change_get_by_id(email_change_id)
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
async fn test_api_email_change_get_by_id() {
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new(0, "test_api_post_like_get_by_post").await;

    let (_user1, session_token1) = server
        .user_add_full("hey", "hey@heyadora.com", "1nnerogGeron@@$")
        .await;
    let (_user2, session_token2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "1nnerogGeron@@$")
        .await;

    server.state.set_email_change_expiration(5).await;

    let result = server.email_change_get_by_id(1, session_token1).await;
    assert!(matches!(result, Err(EmailChangeGetByIdErr::NotFound)));

    let email_change = server.email_change_add(session_token1).await.unwrap();

    let result = server
        .email_change_get_by_id(email_change.id, session_token1)
        .await;
    assert!(matches!(result, Ok(_)));

    let result = server.email_change_get_by_id(1, session_token2).await;
    assert!(matches!(
        result,
        Err(EmailChangeGetByIdErr::Unauthorized(_))
    ));
    server.state.set_time(10);

    let result = server.email_change_get_by_id(1, session_token1).await;
    assert!(matches!(result, Err(EmailChangeGetByIdErr::Expired)));

    server.state.set_time(1);

    server
        .email_change_update_cancel(email_change.id, session_token1)
        .await
        .unwrap();

    let result = server.email_change_get_by_id(1, session_token1).await;
    assert!(matches!(result, Err(EmailChangeGetByIdErr::AlreadyUsed)));
}
