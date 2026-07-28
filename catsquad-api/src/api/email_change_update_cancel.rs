use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeUpdateCancelErr, DbUser};
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

pub fn status_code(result: &Result<EmailChangeRes, EmailChangeUpdateCancelErr>) -> StatusCode {
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
    let time = app.get_time().await;
    let inner = async || -> Result<EmailChangeRes, EmailChangeUpdateCancelErr> {
        let user_id = db_user.id.clone();
        let email_change_key = req.email_change_key.clone();

        let email_change = app
            .db
            .email_change_update_cancel(time, user_id, email_change_key)
            .await
            .map_err(from_db_email_change_update_cancel_err)?;

        Ok(from_db_email_change(email_change))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

// #[cfg(test)]
// mod test_utils {
//     use catsquad_shared::{
//         EmailChangeRes, EmailChangeUpdateCancelErr, EmailChangeUpdateCancelReq,
//         LINK_API_EMAIL_CHANGE_UPDATE_CANCEL, ToForm,
//     };

//     use crate::TestServer;

//     impl TestServer {
//         pub async fn email_change_update_cancel(
//             &self,
//             email_change_key: impl Into<String>,
//             session_key: impl Into<String>,
//         ) -> Result<EmailChangeRes, EmailChangeUpdateCancelErr> {
//             let session_key = session_key.into();
//             let data: String = EmailChangeUpdateCancelReq {
//                 email_change_key: email_change_key.into(),
//             }
//             .to_form()
//             .unwrap();

//             self.post_auth(LINK_API_EMAIL_CHANGE_UPDATE_CANCEL, data, session_key)
//                 .await
//                 .0
//         }
//     }
// }

// #[tokio::test]
// async fn test_email_chang_updatee_cancel() {
//     use catsquad_shared::EmailChangeUpdateFinishErr;

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
//     {
//         let email_change = server.email_change_add(&session_key).await.unwrap();

//         let current_token = server
//             .state
//             .db
//             .email_change_get_by_key(email_change.key.clone())
//             .await
//             .unwrap()
//             .current
//             .token;

//         let email_change = server
//             .email_change_update_current_confirm(
//                 email_change.key.clone(),
//                 current_token.clone(),
//                 &session_key,
//             )
//             .await
//             .unwrap();

//         let email_change = server
//             .email_change_update_new_add(
//                 email_change.key.clone(),
//                 "hey3@heyadora.com",
//                 &session_key,
//             )
//             .await
//             .unwrap();

//         let new_token = server
//             .state
//             .db
//             .email_change_get_by_key(email_change.key.clone())
//             .await
//             .unwrap()
//             .new
//             .unwrap()
//             .token;

//         let email_change = server
//             .email_change_update_new_confirm(email_change.key.clone(), &new_token, &session_key)
//             .await
//             .unwrap();

//         let result = server
//             .email_change_update_cancel("invalid", &session_key)
//             .await;
//         assert!(matches!(result, Err(EmailChangeUpdateCancelErr::NotFound)));

//         let result = server
//             .email_change_update_cancel(email_change.key.clone(), &session_key2)
//             .await;
//         assert!(matches!(
//             result,
//             Err(EmailChangeUpdateCancelErr::Unauthorized(_))
//         ));

//         server.state.set_time(11).await;
//         let result = server
//             .email_change_update_cancel(email_change.key.clone(), &session_key)
//             .await;
//         assert!(matches!(result, Err(EmailChangeUpdateCancelErr::Expired)));
//         server.state.set_time(0).await;

//         let email_change = server
//             .email_change_update_cancel(email_change.key.clone(), &session_key)
//             .await
//             .unwrap();

//         let result = server
//             .email_change_update_finish(email_change.key.clone(), &session_key)
//             .await;
//         assert!(matches!(
//             result,
//             Err(EmailChangeUpdateFinishErr::AlreadyUsed)
//         ));
//     }
// }
