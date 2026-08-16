use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbEmailChangeGetByKeyErr, DbUser, id_to_string};
use catsquad_log::prelude::*;
use catsquad_shared::{EmailChangeRes, EmailChangeResendErr, EmailChangeResendReq};

use crate::{
    api::{
        email_change_add::{from_db_email_change, send_email_email_change_add},
        email_change_update_new_add::send_email_email_change_update_new_add,
    },
    state::AppState,
};

fn from_db_email_change_get_by_key_err(value: DbEmailChangeGetByKeyErr) -> EmailChangeResendErr {
    match value {
        DbEmailChangeGetByKeyErr::EmailChangeNotFound => EmailChangeResendErr::NotFound,
        DbEmailChangeGetByKeyErr::Unauthorized => {
            EmailChangeResendErr::Unauthorized("unauthorized".to_string())
        }
        DbEmailChangeGetByKeyErr::AlreadyUsed => EmailChangeResendErr::AlreadyUsed,
        DbEmailChangeGetByKeyErr::Expired => EmailChangeResendErr::Expired,
        DbEmailChangeGetByKeyErr::Db(_) => EmailChangeResendErr::InternalServer,
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
    let time = app.get_time().await;
    let inner = async || -> Result<EmailChangeRes, EmailChangeResendErr> {
        let user_id = db_user.id.clone();
        let user_email = db_user.email.clone();
        let email_change_key = req.email_change_key.clone();

        let email_change = app
            .db
            .email_change_get_by_key(time, user_id, email_change_key)
            .await
            .map_err(from_db_email_change_get_by_key_err)?;
        let key = id_to_string(email_change.id.clone());
        let address = app.get_address().await;

        if let Some(email_change_new) = email_change.new.as_ref().map(|v| v.clone())
            && !email_change_new.token_used
        {
            let token = email_change_new.token;
            let new_email = email_change_new.email;
            let email_body_new = send_email_email_change_update_new_add(address, key, token);
            let _ = app
                .db
                .email_sent_add(
                    0,
                    catsquad_db::DbEmailSentReason::UserEmailChangeAddNew,
                    new_email,
                    email_body_new,
                )
                .await;
            //
        } else if !email_change.current.token_used {
            let token = email_change.current.token.clone();
            let email_body_current = send_email_email_change_add(address, key, token);
            let _ = app
                .db
                .email_sent_add(
                    0,
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
    use catsquad_shared as cs;

    impl TestServer {
        pub async fn email_change_resend(
            &self,
            email_change_key: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::EmailChangeRes, cs::EmailChangeResendErr> {
            self.client
                .email_change_resend(email_change_key)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[tokio::test]
async fn test_email_change_resend() {
    init_log();
    let server = crate::TestServer::new().await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "a1234567890111GG11$")
        .await;

    let (_user2, session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "a1234567890111GG11$")
        .await;

    server.state.set_time(0).await;
    server.state.set_email_change_expiration(10).await;
    {
        let result = server.email_change_resend("invalid", &session_key).await;

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 0);

        assert_eq!(result, Err(EmailChangeResendErr::NotFound));
        let email_change = server.email_change_add(&session_key).await.unwrap();

        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 1);

        let result = server
            .email_change_resend(email_change.key.clone(), &session_key2)
            .await;
        assert!(matches!(result, Err(EmailChangeResendErr::Unauthorized(_))));

        server.state.set_time(11).await;
        let result = server
            .email_change_resend(email_change.key.clone(), &session_key)
            .await;
        assert!(matches!(result, Err(EmailChangeResendErr::Expired)));
        server.state.set_time(0).await;

        let email_change = server
            .email_change_resend(email_change.key.clone(), &session_key)
            .await
            .unwrap();
        let emails = server
            .email_sent_get_filtered(catsquad_db::DbEmailSentReason::UserEmailChangeAddCurrent)
            .await;
        assert_eq!(emails.len(), 2);

        let current_token = server
            .email_change_get_current_token(0, &user1, email_change.key.clone())
            .await;

        let email_change = server
            .email_change_update_current_confirm(
                email_change.key.clone(),
                current_token,
                &session_key,
            )
            .await
            .unwrap();

        let result = server
            .email_change_resend(email_change.key.clone(), &session_key)
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
            .email_change_update_new_add(
                email_change.key.clone(),
                "prime3@heyadora.com",
                &session_key,
            )
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
            .email_change_resend(email_change.key.clone(), &session_key)
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
            .email_change_get_new_token(0, &user1, email_change.key.clone())
            .await;

        server
            .email_change_update_new_confirm(email_change.key.clone(), new_token, &session_key)
            .await
            .unwrap();

        let result = server
            .email_change_resend(email_change.key.clone(), &session_key)
            .await;
        assert_eq!(result, Err(EmailChangeResendErr::NothingToResend));

        server
            .email_change_update_finish(email_change.key.clone(), &session_key)
            .await
            .unwrap();

        let result = server
            .email_change_resend(email_change.key.clone(), &session_key)
            .await;
        assert_eq!(result, Err(EmailChangeResendErr::AlreadyUsed));
    }
}
