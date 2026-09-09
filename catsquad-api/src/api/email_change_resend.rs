use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeGetByIdErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{EmailChangeRes, EmailChangeResendErr, EmailChangeResendReq};

use crate::{
    api::{
        email_change_add::{from_db_email_change, send_email_email_change_add},
        email_change_update_new_add::send_email_email_change_update_new_add,
    },
    state::AppState,
};

fn from_db_email_change_get_by_key_err(value: DbEmailChangeGetByIdErr) -> EmailChangeResendErr {
    match value {
        DbEmailChangeGetByIdErr::EmailChangeNotFound => EmailChangeResendErr::NotFound,
        DbEmailChangeGetByIdErr::Unauthorized => {
            EmailChangeResendErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeGetByIdErr::AlreadyUsed => EmailChangeResendErr::AlreadyUsed,
        DbEmailChangeGetByIdErr::Expired => EmailChangeResendErr::Expired,
        DbEmailChangeGetByIdErr::Db(_) => EmailChangeResendErr::InternalServer,
    }
}

pub fn status_code(result: &Result<EmailChangeRes, EmailChangeResendErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeResendErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(EmailChangeResendErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeResendErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(EmailChangeResendErr::Expired) => StatusCode::BAD_REQUEST,
        Err(EmailChangeResendErr::NothingToResend) => StatusCode::BAD_REQUEST,
        Err(EmailChangeResendErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn email_change_resend(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<EmailChangeResendReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<EmailChangeRes, EmailChangeResendErr> {
        let user_username = db_user.username.clone();
        let user_email = db_user.email.clone();
        let email_change_id = req.email_change_id.clone();

        let email_change = app
            .db
            .email_change_get_by_id(time, user_username, email_change_id)
            .await
            .map_err(from_db_email_change_get_by_key_err)?;
        let email_change_id = email_change.id;
        let address = app.get_address().await;

        if !email_change.new_email.is_empty() && !email_change.new_used {
            let token = email_change.new_token;
            let new_email = email_change.new_email.clone();
            let email_body_new =
                send_email_email_change_update_new_add(address, email_change_id, token);
            let _ = app
                .db
                .email_sent_add(
                    time,
                    catsquad_db::DbEmailSentReason::UserEmailChangeAddNew,
                    new_email,
                    email_body_new,
                )
                .await;
            //
        } else if !email_change.current_used {
            let token = email_change.current_token.clone();
            let email_body_current = send_email_email_change_add(address, email_change_id, token);
            let _ = app
                .db
                .email_sent_add(
                    time,
                    catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent,
                    user_email,
                    email_body_current,
                )
                .await;
        } else {
            return Err(EmailChangeResendErr::NothingToResend);
        }

        Ok(from_db_email_change(email_change))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn email_change_resend(
            &self,
            email_change_id: i64,
            session_token: Uuid,
        ) -> Result<cs::EmailChangeRes, cs::EmailChangeResendErr> {
            self.client
                .email_change_resend(email_change_id)
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

#[tokio::test]
async fn test_api_email_change_resend() {
    init_log();
    let server = crate::TestServer::new(0, "test_api_email_change_resend").await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "a1234567890111GG11$")
        .await;

    let (_user2, session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "a1234567890111GG11$")
        .await;

    server.state.set_time(0);
    server.state.set_email_change_expiration(10).await;
    {
        let result = server.email_change_resend(0, session_key).await;

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 0);

        assert_eq!(result, Err(EmailChangeResendErr::NotFound));
        let email_change = server.email_change_add(session_key).await.unwrap();

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 1);

        let result = server
            .email_change_resend(email_change.id, session_key2)
            .await;
        assert!(matches!(result, Err(EmailChangeResendErr::Unauthorized(_))));

        server.state.set_time(11);
        let result = server
            .email_change_resend(email_change.id, session_key)
            .await;
        assert!(matches!(result, Err(EmailChangeResendErr::Expired)));
        server.state.set_time(0);

        let email_change = server
            .email_change_resend(email_change.id, session_key)
            .await
            .unwrap();
        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 2);

        let current_token = server
            .email_change_get_current_token(0, &user1, email_change.id)
            .await;

        server
            .email_change_update_current_confirm(email_change.id, current_token, session_key)
            .await
            .unwrap();

        let result = server
            .email_change_resend(email_change.id, session_key)
            .await;
        assert_eq!(result, Err(EmailChangeResendErr::NothingToResend));

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 2);

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddNew)
            .await;
        assert_eq!(emails.len(), 0);

        let email_change = server
            .email_change_update_new_add(email_change.id, "prime3@heyadora.com", session_key)
            .await
            .unwrap();

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 2);

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddNew)
            .await;
        assert_eq!(emails.len(), 1);

        let email_change = server
            .email_change_resend(email_change.id, session_key)
            .await
            .unwrap();

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 2);

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddNew)
            .await;
        assert_eq!(emails.len(), 2);

        let new_token = server
            .email_change_get_new_token(0, &user1, email_change.id)
            .await;

        server
            .email_change_update_new_confirm(email_change.id, new_token, session_key)
            .await
            .unwrap();

        let result = server
            .email_change_resend(email_change.id, session_key)
            .await;
        assert_eq!(result, Err(EmailChangeResendErr::NothingToResend));

        server
            .email_change_update_finish(email_change.id, session_key)
            .await
            .unwrap();

        let result = server
            .email_change_resend(email_change.id, session_key)
            .await;
        assert_eq!(result, Err(EmailChangeResendErr::AlreadyUsed));
    }
}
