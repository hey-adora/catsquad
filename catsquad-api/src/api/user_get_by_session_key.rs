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
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn user_get_by_session_key(
            &self,
            session_token: Uuid,
        ) -> Result<cs::SensitiveUserRes, cs::UserGetBySessionKeyErr> {
            self.client
                .user_get_by_session_key()
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
async fn test_api_user_get_by_sessino_key() {
    use catsquad_log::prelude::*;
    init_log();
    let server = crate::TestServer::new(0, "test_api_user_get_by_sessino_key").await;

    let (user_add, session_key) = server
        .user_add_full("hey", "prime@heyadora.com", "PAss$ord11111")
        .await;
    let user_get = server.user_get_by_session_key(session_key).await.unwrap();
    assert_eq!(user_add.username, user_get.username);
}

#[cfg(test)]
#[tokio::test]
async fn test_api_security_test_user_get_by_sessino_key() {
    use axum::http::header;
    use catsquad_log::prelude::*;

    use crate::auth::{ERR_MSG_COOKIE, ERR_MSG_SESSION, create_auth_cookie_str};

    init_log();
    let server = crate::TestServer::new(0, "test_api_security_test_user_get_by_sessino_key").await;

    let user_get = server
        .client
        .user_get_by_session_key()
        .header_add(header::COOKIE, create_auth_cookie_str(""))
        .send()
        .await
        .into_json()
        .await;
    // let user_get = server.user_get_by_session_key(0_u128.to_be_bytes()).await;
    assert_eq!(
        user_get,
        Err(UserGetBySessionKeyErr::Unauthorized(
            ERR_MSG_COOKIE.to_string()
        ))
    );

    let user_get = server
        .user_get_by_session_key(u128::MAX.to_be_bytes())
        .await;
    assert_eq!(
        user_get,
        Err(UserGetBySessionKeyErr::Unauthorized(
            ERR_MSG_SESSION.to_string()
        ))
    );
}
