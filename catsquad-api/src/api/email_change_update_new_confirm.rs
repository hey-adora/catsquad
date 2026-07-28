use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeUpdateNewConfirmErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    EmailChangeRes, EmailChangeUpdateNewConfirmErr, EmailChangeUpdateNewConfirmReq,
};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn from_db_email_change_update_new_confirm_err(
    value: DbEmailChangeUpdateNewConfirmErr,
) -> EmailChangeUpdateNewConfirmErr {
    match value {
        DbEmailChangeUpdateNewConfirmErr::NotFound => EmailChangeUpdateNewConfirmErr::NotFound,
        DbEmailChangeUpdateNewConfirmErr::Unauthorized => {
            EmailChangeUpdateNewConfirmErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeUpdateNewConfirmErr::AlreadyUsed => {
            EmailChangeUpdateNewConfirmErr::AlreadyUsed
        }
        DbEmailChangeUpdateNewConfirmErr::Expired => EmailChangeUpdateNewConfirmErr::Expired,
        DbEmailChangeUpdateNewConfirmErr::NewEmailNotSet => {
            EmailChangeUpdateNewConfirmErr::NewEmailNotSet
        }
        DbEmailChangeUpdateNewConfirmErr::InvalidToken => {
            EmailChangeUpdateNewConfirmErr::InvalidToken
        }
        DbEmailChangeUpdateNewConfirmErr::Db(_) => EmailChangeUpdateNewConfirmErr::InternalServer,
    }
}

fn status_code(result: &Result<EmailChangeRes, EmailChangeUpdateNewConfirmErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeUpdateNewConfirmErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewConfirmErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateNewConfirmErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewConfirmErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewConfirmErr::NewEmailNotSet) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateNewConfirmErr::InvalidToken) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateNewConfirmErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn email_change_update_new_confirm(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateNewConfirmReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<EmailChangeRes, EmailChangeUpdateNewConfirmErr> {
        let user_id = db_user.id.clone();
        let email_change_key = req.email_change_key.clone();
        let email_change_token = req.token.clone();

        let email_change = app
            .db
            .email_change_update_new_confirm(time, user_id, email_change_key, email_change_token)
            .await
            .map_err(from_db_email_change_update_new_confirm_err)?;

        Ok(from_db_email_change(email_change))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

// #[cfg(test)]
// mod test_utils {
//     use catsquad_shared::{
//         EmailChangeRes, EmailChangeUpdateNewConfirmErr, EmailChangeUpdateNewConfirmReq,
//         LINK_API_EMAIL_CHANGE_UPDATE_NEW_CONFIRM, ToForm,
//     };

//     use crate::TestServer;

//     impl TestServer {
//         pub async fn email_change_update_new_confirm(
//             &self,
//             email_change_key: impl Into<String>,
//             token: impl Into<String>,
//             session_key: impl Into<String>,
//         ) -> Result<EmailChangeRes, EmailChangeUpdateNewConfirmErr> {
//             let session_key = session_key.into();
//             let data: String = EmailChangeUpdateNewConfirmReq {
//                 email_change_key: email_change_key.into(),
//                 token: token.into(),
//             }
//             .to_form()
//             .unwrap();

//             self.post_auth(LINK_API_EMAIL_CHANGE_UPDATE_NEW_CONFIRM, data, session_key)
//                 .await
//                 .0
//         }
//     }
// }

// #[tokio::test]
// async fn test_email_change_update_new_confirm() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let (user1, session_key) = server
//         .user_add_2("hey", "hey@heyadora.com", "1234567890111GG11$")
//         .await;

//     let (user2, session_key2) = server
//         .user_add_2("hey2", "hey2@heyadora.com", "1234567890111GG11$")
//         .await;

//     server.state.set_time(0).await;
//     server.state.set_email_change_expiration(10).await;

//     let email_change = server.email_change_add(&session_key).await.unwrap();

//     let current_token = server
//         .state
//         .db
//         .email_change_get_by_key(email_change.key.clone())
//         .await
//         .unwrap()
//         .current
//         .token;

//     let email_change = server
//         .email_change_update_current_confirm(
//             email_change.key.clone(),
//             current_token.clone(),
//             &session_key,
//         )
//         .await
//         .unwrap();

//     let result = server
//         .email_change_update_new_confirm(email_change.key.clone(), "invalid", &session_key)
//         .await;

//     assert!(matches!(
//         result,
//         Err(EmailChangeUpdateNewConfirmErr::NewEmailNotSet)
//     ));

//     let email_change = server
//         .email_change_update_new_add(email_change.key.clone(), "hey3@heyadora.com", &session_key)
//         .await
//         .unwrap();

//     let new_token = server
//         .state
//         .db
//         .email_change_get_by_key(email_change.key.clone())
//         .await
//         .unwrap()
//         .new
//         .unwrap()
//         .token;

//     let result = server
//         .email_change_update_new_confirm("invalid", "invalid", &session_key)
//         .await;

//     assert!(matches!(
//         result,
//         Err(EmailChangeUpdateNewConfirmErr::NotFound)
//     ));

//     let result = server
//         .email_change_update_new_confirm(email_change.key.clone(), "invalid", &session_key)
//         .await;

//     assert!(matches!(
//         result,
//         Err(EmailChangeUpdateNewConfirmErr::InvalidToken)
//     ));

//     server.state.set_time(11).await;

//     let result = server
//         .email_change_update_new_confirm(email_change.key.clone(), &current_token, &session_key)
//         .await;

//     assert!(matches!(
//         result,
//         Err(EmailChangeUpdateNewConfirmErr::Expired)
//     ));
//     server.state.set_time(0).await;

//     let result = server
//         .email_change_update_new_confirm(email_change.key.clone(), &current_token, &session_key)
//         .await;

//     assert!(matches!(
//         result,
//         Err(EmailChangeUpdateNewConfirmErr::InvalidToken)
//     ));

//     let email_change = server
//         .email_change_update_new_confirm(email_change.key.clone(), &new_token, &session_key)
//         .await
//         .unwrap();

//     let result = server
//         .email_change_update_new_confirm(email_change.key.clone(), &new_token, &session_key)
//         .await;

//     assert!(matches!(
//         result,
//         Err(EmailChangeUpdateNewConfirmErr::AlreadyUsed)
//     ));
// }
