use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{
    DbEmailSent, DbEmailSentReason, DbPasswordChangeAddErr, DbPasswordChangeUpdateConfirmErr,
    DbUser, id_to_string,
};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PasswordChangeUpdateConfirmErr, PasswordChangeUpdateConfirmReq, PasswordChangeUpdateConfirmRes,
    validate_password,
};

use crate::{
    auth::{hash_password, verify_password},
    state::AppState,
};

fn from_db_password_change_confirm_err(
    err: DbPasswordChangeUpdateConfirmErr,
) -> PasswordChangeUpdateConfirmErr {
    match err {
        DbPasswordChangeUpdateConfirmErr::Expired => PasswordChangeUpdateConfirmErr::Expired,
        DbPasswordChangeUpdateConfirmErr::AlreadyUsed => {
            PasswordChangeUpdateConfirmErr::AlreadyUsed
        }
        DbPasswordChangeUpdateConfirmErr::PasswordKeyNotFound => {
            PasswordChangeUpdateConfirmErr::PasswordKeyNotFound
        }
        DbPasswordChangeUpdateConfirmErr::Db(_) => PasswordChangeUpdateConfirmErr::InternalServer,
    }
}

fn status_code(
    result: &Result<PasswordChangeUpdateConfirmRes, PasswordChangeUpdateConfirmErr>,
) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PasswordChangeUpdateConfirmErr::NewPasswordInvalid) => StatusCode::BAD_REQUEST,
        Err(PasswordChangeUpdateConfirmErr::PasswordKeyNotFound) => StatusCode::BAD_REQUEST,
        Err(PasswordChangeUpdateConfirmErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(PasswordChangeUpdateConfirmErr::Expired) => StatusCode::BAD_REQUEST,
        Err(PasswordChangeUpdateConfirmErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(PasswordChangeUpdateConfirmErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_password_change_confirm(address: impl AsRef<str>) -> String {
    let link = "password was changed".to_string();
    debug!("EMAIL SENT {link}");
    link
}

fn send_email_password_reset_confirm(address: impl AsRef<str>) -> String {
    // let link = link_absolute_reg_finish(address, token);
    let link = "password was reset".to_string();
    debug!("EMAIL SENT {link}");
    link
}

pub async fn user_password_change_confirm(
    db_user: Extension<Option<DbUser>>,
    State(app): State<AppState>,
    Form(req): Form<PasswordChangeUpdateConfirmReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner =
        async || -> Result<PasswordChangeUpdateConfirmRes, PasswordChangeUpdateConfirmErr> {
            validate_password(&req.new_password)
                .map_err(|_err| PasswordChangeUpdateConfirmErr::NewPasswordInvalid)?;
            let new_password = hash_password(&req.new_password)
                .map_err(|_err| PasswordChangeUpdateConfirmErr::InternalServer)?;

            let password_change = app
                .db
                .password_change_update_confirm(time, req.password_change_key, new_password)
                .await
                .map_err(from_db_password_change_confirm_err)?;
            let email = password_change.user.email;

            let address = app.get_address().await;

            if let Some(db_user) = &*db_user {
                let email_body = send_email_password_change_confirm(address);
                let _ = app
                    .db
                    .email_sent_add(
                        0,
                        catsquad_db::DbEmailSentReason::UserPasswordChangeConfirm,
                        email,
                        email_body,
                    )
                    .await;
            } else {
                let email_body = send_email_password_reset_confirm(address);
                let _ = app
                    .db
                    .email_sent_add(
                        0,
                        catsquad_db::DbEmailSentReason::UserPasswordResetConfirm,
                        email,
                        email_body,
                    )
                    .await;
            };

            Ok(PasswordChangeUpdateConfirmRes {})
        };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

// #[cfg(test)]
// mod test_utils {

//     use axum::http::StatusCode;
//     use catsquad_db::{DbPasswordChange, id_to_string};
//     use catsquad_shared::{
//         LINK_API_PASSWORD_CHANGE_UPDATE_CONFIRM, PasswordChangeUpdateConfirmErr,
//         PasswordChangeUpdateConfirmReq, PasswordChangeUpdateConfirmRes, ToForm,
//     };

//     use crate::TestServer;

//     impl TestServer {
//         pub async fn user_password_change_confirm(
//             &self,
//             password_change_key: impl Into<String>,
//             new_password: impl Into<String>,
//             session_key: impl Into<String>,
//         ) -> Result<PasswordChangeUpdateConfirmRes, PasswordChangeUpdateConfirmErr> {
//             let session_key = session_key.into();
//             let data = PasswordChangeUpdateConfirmReq {
//                 password_change_key: password_change_key.into(),
//                 new_password: new_password.into(),
//             }
//             .to_form()
//             .unwrap();
//             self.post_auth(LINK_API_PASSWORD_CHANGE_UPDATE_CONFIRM, data, session_key)
//                 .await
//                 .0
//         }

//         pub async fn user_password_change(
//             &self,
//             email: impl Into<String>,
//             new_password: impl Into<String>,
//             session_key: impl Into<String>,
//         ) -> DbPasswordChange {
//             let email = email.into();
//             let session_key = session_key.into();
//             let new_password = new_password.into();

//             let (result, status) = self.password_change_add(&email, &session_key).await;
//             let password_change = self
//                 .state
//                 .db
//                 .password_change_get_all()
//                 .await
//                 .unwrap()
//                 .into_iter()
//                 .find(|v| v.user.email == email && !v.used)
//                 .unwrap();

//             let result = self
//                 .user_password_change_confirm(
//                     id_to_string(password_change.id.clone()),
//                     new_password,
//                     session_key,
//                 )
//                 .await
//                 .unwrap();

//             let password_change = self
//                 .state
//                 .db
//                 .password_change_get_all()
//                 .await
//                 .unwrap()
//                 .into_iter()
//                 .find(|v| v.id == password_change.id)
//                 .unwrap();

//             password_change
//         }
//     }
// }

// #[tokio::test]
// async fn test_user_password_change_confirm() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let (user, session_key) = server
//         .user_add_2("hey", "hey@heyadora.com", "hello1111111@1P")
//         .await;
//     let (user2, session_key2) = server
//         .user_add_2("hey2", "hey2@heyadora.com", "hello1111111@1P")
//         .await;

//     let result = server.user_get_by_session_key(session_key.clone()).await.0;
//     assert!(result.is_ok());

//     // password change
//     let result = server
//         .user_password_change("hey@heyadora.com", "hello1111111@2P", session_key.clone())
//         .await;

//     let result = verify_password("hello1111111@2P", result.user.password);
//     assert!(result.is_ok());
//     let emails = server
//         .email_sent_get_filtered(DbEmailSentReason::UserPasswordChangeConfirm)
//         .await;

//     assert_eq!(emails.len(), 1);
//     assert_eq!(
//         emails[0].reason,
//         DbEmailSentReason::UserPasswordChangeConfirm.to_string()
//     );

//     let result = server.user_get_by_session_key(session_key.clone()).await.0;
//     assert!(result.is_err());

//     // password reset

//     let result = server
//         .user_password_change("hey@heyadora.com", "hello1111111@3P", "invalid")
//         .await;

//     let result = verify_password("hello1111111@3P", result.user.password);
//     assert!(result.is_ok());

//     let emails = server
//         .email_sent_get_filtered(DbEmailSentReason::UserPasswordResetConfirm)
//         .await;

//     assert_eq!(emails.len(), 1);
//     assert_eq!(
//         emails[0].reason,
//         DbEmailSentReason::UserPasswordResetConfirm.to_string()
//     );

//     let result = server.user_get_by_session_key(session_key.clone()).await.0;
//     assert!(result.is_err());

//     let result = server.user_get_by_session_key(session_key2.clone()).await.0;
//     assert!(result.is_ok());
// }
