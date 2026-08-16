use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use catsquad_db::DbUser;
use catsquad_shared::{SensitiveUserRes, UserGetBySessionKeyErr};

use crate::api::user_add::from_db_user_sensitive;

fn status_code(result: &Result<SensitiveUserRes, UserGetBySessionKeyErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserGetBySessionKeyErr::Unauthorized(_)) => StatusCode::OK,
        Err(UserGetBySessionKeyErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_get_by_session_token(db_user: Extension<DbUser>) -> impl IntoResponse {
    let db_user = (*db_user).clone();
    let result = Ok::<SensitiveUserRes, UserGetBySessionKeyErr>(from_db_user_sensitive(db_user));
    let status_code = status_code(&result);
    (status_code, Json(result))
}

#[cfg(any(test, feature = "test_server"))]
mod test_utils {
    use axum::http::header;
    use catsquad_shared as cs;

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn user_get_by_session_key(
            &self,
            session_key: impl Into<String>,
        ) -> Result<cs::SensitiveUserRes, cs::UserGetBySessionKeyErr> {
            self.client
                .user_get_by_session_key()
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
async fn test_user_get_by_sessino_key() {
    use catsquad_db::id_to_string;
    use catsquad_log::prelude::*;
    init_log();
    let server = crate::TestServer::new().await;

    let (user_add, session_key) = server
        .user_add_full("hey", "prime@heyadora.com", "PAss$ord11111")
        .await;
    let user_get = server.user_get_by_session_key(session_key).await.unwrap();
    assert_eq!(id_to_string(user_add.id), user_get.key);
}

#[cfg(test)]
#[tokio::test]
async fn security_test_user_get_by_sessino_key() {
    use catsquad_log::prelude::*;

    use crate::auth::{ERR_MSG_COOKIE, ERR_MSG_SESSION};

    init_log();
    let server = crate::TestServer::new().await;

    let user_get = server.user_get_by_session_key("INVALID").await;
    assert_eq!(
        user_get,
        Err(UserGetBySessionKeyErr::Unauthorized(
            ERR_MSG_COOKIE.to_string()
        ))
    );

    let user_get = server.user_get_by_session_key("y4lu28oeddllera6275b").await;
    assert_eq!(
        user_get,
        Err(UserGetBySessionKeyErr::Unauthorized(
            ERR_MSG_SESSION.to_string()
        ))
    );
}
