use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeUpdateNewAddErr, DbEmailSentReason, DbUser, id_to_string};
use catsquad_log::prelude::*;
use catsquad_shared::{EmailChangeNewAddErr, EmailChangeRes, EmailChangeUpdateNewAddReq};

use crate::{api::email_change_add::from_db_email_change, state::AppState};

fn from_db_email_change_update_new_add_err(
    value: DbEmailChangeUpdateNewAddErr,
) -> EmailChangeNewAddErr {
    match value {
        DbEmailChangeUpdateNewAddErr::NotFound => EmailChangeNewAddErr::NotFound,
        DbEmailChangeUpdateNewAddErr::Unauthorized => {
            EmailChangeNewAddErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeUpdateNewAddErr::AlreadyUsed => EmailChangeNewAddErr::AlreadyUsed,
        DbEmailChangeUpdateNewAddErr::Expired => EmailChangeNewAddErr::Expired,
        DbEmailChangeUpdateNewAddErr::NotConfirmed => EmailChangeNewAddErr::NotConfirmed,
        DbEmailChangeUpdateNewAddErr::EmailIsTaken(v) => EmailChangeNewAddErr::EmailIsTaken(v),
        DbEmailChangeUpdateNewAddErr::Db(_) => EmailChangeNewAddErr::InternalServer,
    }
}

fn status_code(result: &Result<EmailChangeRes, EmailChangeNewAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeNewAddErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeNewAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeNewAddErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeNewAddErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeNewAddErr::NotConfirmed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeNewAddErr::EmailIsTaken(_)) => StatusCode::BAD_REQUEST,
        Err(EmailChangeNewAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_email_change_update_new_add(
    address: impl Into<String>,
    email_change_key: impl Into<String>,
    token: impl Into<String>,
) -> String {
    // let link = link_absolute_reg_finish(address, token);
    let link = "placeholder change".to_string();
    debug!("EMAIL SENT {link}");
    link
}

pub async fn email_change_update_new_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeUpdateNewAddReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<EmailChangeRes, EmailChangeNewAddErr> {
        let user_id = db_user.id.clone();
        let user_email = db_user.email.clone();
        let email_change_key = req.email_change_key;
        let new_email = req.new_email;

        let email_change = app
            .db
            .email_change_update_new_add(time, user_id, email_change_key, new_email)
            .await
            .map_err(from_db_email_change_update_new_add_err)?;
        let token = email_change.current.token.clone();
        let key = id_to_string(email_change.id.clone());

        let address = app.get_address().await;
        let email_body = send_email_email_change_update_new_add(address, key, token);
        let _ = app
            .db
            .email_sent_add(
                0,
                DbEmailSentReason::UserEmailChangeAddNew,
                user_email,
                email_body,
            )
            .await;

        Ok(from_db_email_change(email_change))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

// #[cfg(test)]
// mod test_utils {
//     use catsquad_shared::{
//         EmailChangeNewAddErr, EmailChangeRes, EmailChangeUpdateNewAddReq,
//         LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_ADD, LINK_API_EMAIL_CHANGE_UPDATE_NEW_ADD, ToForm,
//     };

//     use crate::TestServer;

//     impl TestServer {
//         pub async fn email_change_update_new_add(
//             &self,
//             email_change_key: impl Into<String>,
//             new_email: impl Into<String>,
//             session_key: impl Into<String>,
//         ) -> Result<EmailChangeRes, EmailChangeNewAddErr> {
//             let session_key = session_key.into();
//             let data: String = EmailChangeUpdateNewAddReq {
//                 email_change_key: email_change_key.into(),
//                 new_email: new_email.into(),
//             }
//             .to_form()
//             .unwrap();
//             self.post_auth(LINK_API_EMAIL_CHANGE_UPDATE_NEW_ADD, data, session_key)
//                 .await
//                 .0
//         }
//     }
// }

// #[tokio::test]
// async fn test_email_change_update_new_add() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let (user1, session_key) = server
//         .user_add_2("hey", "hey@heyadora.com", "1234567890111GG11$")
//         .await;
//     let (user5, session_key5) = server
//         .user_add_2("hey5", "hey5@heyadora.com", "1234567890111GG11$")
//         .await;
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

//     let result = server
//         .email_change_update_new_add(email_change.key.clone(), "hey2@heyadora.com", &session_key)
//         .await;
//     assert!(matches!(result, Err(EmailChangeNewAddErr::NotConfirmed)));

//     let email_change = server
//         .email_change_update_current_confirm(
//             email_change.key.clone(),
//             current_token.clone(),
//             &session_key,
//         )
//         .await
//         .unwrap();

//     let result = server
//         .email_change_update_new_add("", "hey2@heyadora.com", &session_key)
//         .await;
//     assert!(matches!(result, Err(EmailChangeNewAddErr::NotFound)));

//     let result = server
//         .email_change_update_new_add("", "hey2@heyadora.com", "")
//         .await;
//     assert!(matches!(result, Err(EmailChangeNewAddErr::Unauthorized(_))));

//     server.state.set_time(11).await;
//     let result = server
//         .email_change_update_new_add(email_change.key.clone(), "hey2@heyadora.com", &session_key)
//         .await;
//     assert!(matches!(result, Err(EmailChangeNewAddErr::Expired)));
//     server.state.set_time(0).await;

//     let result = server
//         .email_change_update_new_add(email_change.key.clone(), "hey5@heyadora.com", &session_key)
//         .await;
//     assert!(matches!(result, Err(EmailChangeNewAddErr::EmailIsTaken(_))));

//     let email_change = server
//         .email_change_update_new_add(email_change.key.clone(), "hey2@heyadora.com", &session_key)
//         .await
//         .unwrap();

//     let result = server
//         .email_change_update_new_add(email_change.key.clone(), "hey2@heyadora.com", &session_key)
//         .await;
//     assert!(matches!(result, Err(EmailChangeNewAddErr::AlreadyUsed)));
// }
