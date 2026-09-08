use axum::{
    Form, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use catsquad_db::{DbSession, DbSessionAddErr, DbUser, DbUserGetByEmailErr};
use catsquad_log::prelude::*;
use catsquad_shared::{SessionAddErr, SessionAddReq, SessionRes};

use crate::{
    auth::{create_auth_cookie, verify_password},
    state::AppState,
};

fn from_db_session(value: DbUser) -> SessionRes {
    SessionRes {
        // key: id_to_string(value.user.id),
        username: value.username,
        email: value.email,
        created_at: value.created_at,
    }
}

fn from_db_get_by_email_err(value: DbUserGetByEmailErr) -> SessionAddErr {
    match value {
        DbUserGetByEmailErr::NotFound => SessionAddErr::InvalidCredentials,
        DbUserGetByEmailErr::Db(_) => SessionAddErr::InternalServer,
    }
}

fn from_db_session_add_err(value: DbSessionAddErr) -> SessionAddErr {
    match value {
        DbSessionAddErr::UserNotFound(_) => SessionAddErr::InvalidCredentials,
        DbSessionAddErr::Db(_) => SessionAddErr::InternalServer,
    }
}

fn status_code(result: &Result<SessionRes, SessionAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(SessionAddErr::InvalidCredentials) => StatusCode::UNAUTHORIZED,
        Err(SessionAddErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(SessionAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_session_add(address: impl AsRef<str>) -> String {
    let email = "EMAIL SENT NEW LOGIN FROM FIREFOX".to_string();
    debug!("EMAIL SENT {email}");
    email
}

pub async fn session_add(
    State(app): State<AppState>,
    Form(req): Form<SessionAddReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<(DbUser, DbSession), SessionAddErr> {
        let email = req.email.trim().to_lowercase();
        let password = req.password;

        let user = app
            .db
            .user_get_by_email(&email)
            .await
            .map_err(from_db_get_by_email_err)?;

        verify_password(password, &user.password).map_err(|_| SessionAddErr::InvalidCredentials)?;

        let session = app
            .db
            .session_add(time, &user.email)
            .await
            .map_err(from_db_session_add_err)?;

        let address = app.get_address().await;
        let email_body = send_email_session_add(address);
        let _ = app
            .db
            .email_sent_add(
                time,
                catsquad_db::DbEmailSentReason::InviteAdd,
                email,
                email_body,
            )
            .await;

        Ok((user, session))
    };

    let result = inner().await;
    match result {
        Ok((user, session)) => {
            let headers = create_auth_cookie(session.token);
            let result = Ok(from_db_session(user));
            let status_code = status_code(&result);
            (status_code, headers, Json(result))
        }
        Err(err) => {
            let headers = HeaderMap::new();
            let result = Err(err);
            let status_code = status_code(&result);
            (status_code, headers, Json(result))
        }
    }
}

#[cfg(any(test, feature = "test_server"))]
mod test_utils {
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, str_to_uuid};

    use crate::{TestServer, auth::auth_token_get};

    impl TestServer {
        pub async fn session_add(
            &self,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> Result<cs::SensitiveUserRes, cs::SessionAddErr> {
            self.client
                .session_add(email, password)
                .send()
                .await
                .into_json()
                .await
        }
        pub async fn session_add_with_key(
            &self,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> (cs::SensitiveUserRes, Uuid) {
            let res = self.client.session_add(email, password).send().await;
            let headers = res.get_headers().unwrap();
            let session_token = auth_token_get(&headers, header::SET_COOKIE).unwrap();
            let session_token = str_to_uuid(session_token);
            let res = res.into_json().await.unwrap();
            (res, session_token)
        }
    }
}
// #[cfg(test)]
// mod test_utils {
//     use catsquad_shared::{LINK_API_SESSION_ADD, SessionAddErr, SessionAddReq, SessionRes, ToForm};

//     use crate::TestServer;

//     impl TestServer {
//         pub async fn session_add(
//             &self,
//             email: impl Into<String>,
//             password: impl Into<String>,
//         ) -> (Result<SessionRes, SessionAddErr>, Option<String>) {
//             let data = SessionAddReq {
//                 email: email.into(),
//                 password: password.into(),
//             }
//             .to_form()
//             .unwrap();
//             self.post_and_get_auth_token(LINK_API_SESSION_ADD, data)
//                 .await
//         }
//     }
// }

#[tokio::test]
async fn test_api_session_add() {
    init_log();
    let server = crate::TestServer::new(0, "test_api_session_add").await;

    let email = "hey@heyadora.com";
    let password = "1nnerogGeron@@$";

    let result = server
        .session_add("hey@heyadora.com", "1nnerogGeron@@$")
        .await;
    assert!(matches!(result, Err(SessionAddErr::InvalidCredentials)));
    let result = server.session_add("hey@heyadora.com", "").await;
    assert!(matches!(result, Err(SessionAddErr::InvalidCredentials)));

    let (user, session_key) = server.user_add_full("hey", email, password).await;
    let (session, session_key2) = server.session_add_with_key(email, password).await;
    assert_ne!(session_key, session_key2);
    let user = server.user_get_by_session_key(session_key2).await.unwrap();
    assert_eq!(user.email, email);
}
