use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeConfirmUpdateCurrentErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    EmailChangeRes, EmailChangeUpdateCurrentConfirmErr, EmailChangeUpdateCurrentConfirmReq,
};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn from_db_email_change_confirm_current_err(
    value: DbEmailChangeConfirmUpdateCurrentErr,
) -> EmailChangeUpdateCurrentConfirmErr {
    match value {
        DbEmailChangeConfirmUpdateCurrentErr::NotFound => {
            EmailChangeUpdateCurrentConfirmErr::NotFound
        }
        DbEmailChangeConfirmUpdateCurrentErr::Unauthorized => {
            EmailChangeUpdateCurrentConfirmErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeConfirmUpdateCurrentErr::AlreadyUsed => {
            EmailChangeUpdateCurrentConfirmErr::AlreadyUsed
        }
        DbEmailChangeConfirmUpdateCurrentErr::Expired => {
            EmailChangeUpdateCurrentConfirmErr::Expired
        }
        DbEmailChangeConfirmUpdateCurrentErr::InvalidToken => {
            EmailChangeUpdateCurrentConfirmErr::InvalidToken
        }
        DbEmailChangeConfirmUpdateCurrentErr::Db(_) => {
            EmailChangeUpdateCurrentConfirmErr::InternalServer
        }
    }
}

pub fn status_code(
    result: &Result<EmailChangeRes, EmailChangeUpdateCurrentConfirmErr>,
) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeUpdateCurrentConfirmErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCurrentConfirmErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateCurrentConfirmErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCurrentConfirmErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeUpdateCurrentConfirmErr::InvalidToken) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateCurrentConfirmErr::InternalServer) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn email_change_update_current_confirm(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateCurrentConfirmReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<EmailChangeRes, EmailChangeUpdateCurrentConfirmErr> {
        let user_id = db_user.id.clone();
        let email_change_key = req.email_change_key.clone();
        let email_change_token = req.token.clone();

        let email_change = app
            .db
            .email_change_confirm_update_current(
                time,
                user_id,
                email_change_key,
                email_change_token,
            )
            .await
            .map_err(from_db_email_change_confirm_current_err)?;

        Ok(from_db_email_change(email_change))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

// #[cfg(test)]
// mod test_utils {
//     use catsquad_shared::{
//         EmailChangeRes, EmailChangeUpdateCurrentConfirmErr, EmailChangeUpdateCurrentConfirmReq,
//         LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_CONFIRM, ToForm,
//     };

//     use crate::TestServer;

//     impl TestServer {
//         pub async fn email_change_update_current_confirm(
//             &self,
//             email_change_key: impl Into<String>,
//             token: impl Into<String>,
//             session_key: impl Into<String>,
//         ) -> Result<EmailChangeRes, EmailChangeUpdateCurrentConfirmErr> {
//             let session_key = session_key.into();
//             let data: String = EmailChangeUpdateCurrentConfirmReq {
//                 email_change_key: email_change_key.into(),
//                 token: token.into(),
//             }
//             .to_form()
//             .unwrap();

//             self.post_auth(
//                 LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_CONFIRM,
//                 data,
//                 session_key,
//             )
//             .await
//             .0
//         }
//     }
// }

// #[tokio::test]
// async fn test_email_change_update_current_confirm() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let (user1, session_key) = server
//         .user_add_2("hey", "hey@heyadora.com", "1234567890111GG11$")
//         .await;

//     let (user2, session_key2) = server
//         .user_add_2("hey2", "hey2@heyadora.com", "1234567890111GG11$")
//         .await;

//     {
//         server.state.set_email_change_expiration(10).await;

//         let email_change = server.email_change_add(&session_key).await.unwrap();
//         let current_token = server
//             .state
//             .db
//             .email_change_get_by_key(email_change.key.clone())
//             .await
//             .unwrap()
//             .current
//             .token;

//         let result = server
//             .email_change_update_current_confirm("invalid", "", &session_key)
//             .await;

//         assert!(matches!(
//             result,
//             Err(EmailChangeUpdateCurrentConfirmErr::NotFound)
//         ));

//         let result = server
//             .email_change_update_current_confirm(email_change.key.clone(), "", &session_key)
//             .await;

//         assert!(matches!(
//             result,
//             Err(EmailChangeUpdateCurrentConfirmErr::InvalidToken)
//         ));

//         let result = server
//             .email_change_update_current_confirm(email_change.key.clone(), "", &session_key2)
//             .await;

//         assert!(matches!(
//             result,
//             Err(EmailChangeUpdateCurrentConfirmErr::Unauthorized(_))
//         ));

//         server.state.set_time(11).await;
//         let result = server
//             .email_change_update_current_confirm(
//                 email_change.key.clone(),
//                 current_token.clone(),
//                 &session_key,
//             )
//             .await;

//         assert!(matches!(
//             result,
//             Err(EmailChangeUpdateCurrentConfirmErr::Expired)
//         ));
//         server.state.set_time(0).await;

//         let email_change = server
//             .email_change_update_current_confirm(
//                 email_change.key.clone(),
//                 current_token.clone(),
//                 &session_key,
//             )
//             .await
//             .unwrap();

//         let result = server
//             .email_change_update_current_confirm(
//                 email_change.key.clone(),
//                 current_token.clone(),
//                 &session_key,
//             )
//             .await;

//         assert!(matches!(
//             result,
//             Err(EmailChangeUpdateCurrentConfirmErr::AlreadyUsed)
//         ));
//     }
// }
