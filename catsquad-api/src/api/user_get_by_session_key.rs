use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbUser, id_to_string};
use catsquad_log::prelude::*;
use catsquad_shared::{SensitiveUserRes, UserGetBySessionKeyErr};

use crate::{
    api::user_add::from_db_user_sensitive,
    auth::{ERR_MSG_COOKIE, ERR_MSG_SESSION},
};

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

// #[cfg(test)]
// mod test_utils {
//     use axum::http::StatusCode;
//     use catsquad_shared::LINK_API_SESSION_GET_BY_SESSION_KEY;

//     use crate::{
//         TestServer,
//         api::user_get_by_session_key::{UserGetBySessionKeyErr, UserGetBySessionKeyRes},
//     };

//     impl TestServer {
//         pub async fn user_get_by_session_key(
//             &self,
//             session_key: impl AsRef<str>,
//         ) -> (
//             Result<UserGetBySessionKeyRes, UserGetBySessionKeyErr>,
//             StatusCode,
//         ) {
//             self.get_auth::<Result<UserGetBySessionKeyRes, UserGetBySessionKeyErr>>(
//                 LINK_API_SESSION_GET_BY_SESSION_KEY,
//                 session_key,
//             )
//             .await
//         }
//     }
// }

// #[tokio::test]
// async fn test_user_get_by_sessino_key() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     server.invite_add("prime@heyadora.com").await.unwrap();

//     let invite_token = id_to_string(
//         server.state.db.invite_get_all().await.unwrap()[0]
//             .id
//             .clone(),
//     );
//     let (user_add, session_key) = server.user_add("hey", "PAss$ord11111", invite_token).await;
//     let (user_get, status) = server.user_get_by_session_key(session_key.unwrap()).await;
//     assert_eq!(status, StatusCode::OK);
//     assert_eq!(user_add.unwrap().key, user_get.unwrap().key);
// }

// #[tokio::test]
// async fn security_test_user_get_by_sessino_key() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let (user_get, status) = server.user_get_by_session_key("INVALID").await;
//     assert_eq!(status, StatusCode::UNAUTHORIZED);
//     assert_eq!(
//         user_get,
//         Err(UserGetBySessionKeyErr::Unauthorized(
//             ERR_MSG_COOKIE.to_string()
//         ))
//     );

//     let (user_get, status) = server.user_get_by_session_key("y4lu28oeddllera6275b").await;
//     assert_eq!(status, StatusCode::UNAUTHORIZED);
//     assert_eq!(
//         user_get,
//         Err(UserGetBySessionKeyErr::Unauthorized(
//             ERR_MSG_SESSION.to_string()
//         ))
//     );
// }
