use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{
    DbEmailChange, DbEmailChangeAddErr, DbEmailChangeToken, DbEmailSentReason, DbUser, id_to_string,
};
use catsquad_log::prelude::*;
use catsquad_shared::{
    EmailChangeRes, EmailChangeToken, EmailChangeUpdateCurrentAddErr,
    EmailChangeUpdateCurrentAddReq,
};

use crate::state::AppState;

fn from_db_email_change_add_err(value: DbEmailChangeAddErr) -> EmailChangeUpdateCurrentAddErr {
    match value {
        DbEmailChangeAddErr::UserNotFound => EmailChangeUpdateCurrentAddErr::InternalServer,
        DbEmailChangeAddErr::Db(_) => EmailChangeUpdateCurrentAddErr::InternalServer,
    }
}

pub fn from_db_email_change(value: DbEmailChange) -> EmailChangeRes {
    EmailChangeRes {
        key: id_to_string(value.id),
        current: from_db_email_change_token(value.current),
        new: value.new.map(from_db_email_change_token),
        completed: value.completed,
        expires: value.expires,
        modified_at: value.modified_at,
        created_at: value.created_at,
    }
}

fn from_db_email_change_token(value: DbEmailChangeToken) -> EmailChangeToken {
    EmailChangeToken {
        email: value.email,
        token_used: value.token_used,
    }
}

pub fn status_code(result: &Result<EmailChangeRes, EmailChangeUpdateCurrentAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeUpdateCurrentAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeUpdateCurrentAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn send_email_email_change_add(
    address: impl Into<String>,
    email_change_key: impl Into<String>,
    token: impl Into<String>,
) -> String {
    // let link = link_absolute_reg_finish(address, token);
    let link = "placeholder change".to_string();
    debug!("EMAIL SENT {link}");
    link
}

pub async fn email_change_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let email_change_expiration = app.get_email_change_expiration().await;
    let inner = async || -> Result<EmailChangeRes, EmailChangeUpdateCurrentAddErr> {
        let expires = time + email_change_expiration;
        let user_id = db_user.id.clone();
        let user_email = db_user.email.clone();

        let email_change = app
            .db
            .email_change_add(time, user_id, expires)
            .await
            .map_err(from_db_email_change_add_err)?;
        let token = email_change.current.token.clone();
        let key = id_to_string(email_change.id.clone());

        let address = app.get_address().await;
        let email_body = send_email_email_change_add(address, key, token);
        let _ = app
            .db
            .email_sent_add(
                0,
                catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent,
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
//         EmailChangeRes, EmailChangeUpdateCurrentAddErr, LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_ADD,
//     };

//     use crate::TestServer;

//     impl TestServer {
//         pub async fn email_change_add(
//             &self,
//             session_key: impl Into<String>,
//         ) -> Result<EmailChangeRes, EmailChangeUpdateCurrentAddErr> {
//             let session_key = session_key.into();
//             self.post_auth_empty::<Result<EmailChangeRes, EmailChangeUpdateCurrentAddErr>>(
//                 LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_ADD,
//                 session_key,
//             )
//             .await
//             .0
//         }
//     }
// }

// #[tokio::test]
// async fn test_email_change_add() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let (user1, session_key) = server
//         .user_add_2("hey", "hey@heyadora.com", "1234567890111GG11$")
//         .await;

//     let result = server.email_change_add("invalid").await;
//     assert!(matches!(
//         result,
//         Err(EmailChangeUpdateCurrentAddErr::Unauthorized(_))
//     ));

//     let email_change = server.email_change_add(&session_key).await.unwrap();

//     let emails = server
//         .email_sent_get_filtered(DbEmailSentReason::UserEmailChangeAddCurrent)
//         .await;
//     assert_eq!(emails.len(), 1);
//     assert_eq!(
//         emails[0].reason,
//         DbEmailSentReason::UserEmailChangeAddCurrent.to_string()
//     );
// }
