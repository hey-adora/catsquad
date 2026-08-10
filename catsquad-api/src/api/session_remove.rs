use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::DbSessionRemoveErr;
use catsquad_log::prelude::*;
use catsquad_shared::{SessionDeleteErr, SessionDeleteRes};

use crate::{
    auth::{SessionKey, create_auth_cookie, create_deleted_cookie, verify_password},
    state::AppState,
};

fn from_db_session_remove_err(value: DbSessionRemoveErr) -> SessionDeleteErr {
    match value {
        DbSessionRemoveErr::Db(_) => SessionDeleteErr::InternalServer,
    }
}

fn status_code(result: &Result<SessionDeleteRes, SessionDeleteErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(SessionDeleteErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(SessionDeleteErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn session_remove(
    State(app): State<AppState>,
    session_key: Extension<SessionKey>,
) -> impl IntoResponse {
    let inner = async || -> Result<SessionDeleteRes, SessionDeleteErr> {
        app.db
            .session_remove(session_key.to_string())
            .await
            .map_err(from_db_session_remove_err)?;

        Ok(SessionDeleteRes {})
    };
    let result = inner().await;
    let status_code = status_code(&result);
    let result = Json(result);
    let headers = create_deleted_cookie();
    (status_code, headers, result)
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared as cs;

    impl TestServer {
        pub async fn session_remove(
            &self,
            session_key: impl AsRef<str>,
        ) -> Result<cs::SessionDeleteRes, cs::SessionDeleteErr> {
            self.client
                .session_remove()
                .header_add(header::COOKIE, create_auth_cookie_str(session_key))
                .send()
                .await
                .into_json()
                .await
        }
    }
}
#[tokio::test]
async fn test_session_remove() {
    init_log();
    let server = crate::TestServer::new().await;

    let email = "hey@heyadora.com";
    let password = "1nnerogGeron@@$";

    let (user, session_key) = server.user_add_full("hey", email, password).await;

    let result = server.user_get_by_session_key(&session_key).await;
    assert!(result.is_ok());

    let _result = server.session_remove(&session_key).await.unwrap();

    let result = server.user_get_by_session_key(&session_key).await;
    assert!(result.is_err());
}
