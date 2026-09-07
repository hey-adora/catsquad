use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{
    DbEmailSent, DbEmailSentReason, DbPasswordChangeAddErr, DbPasswordChangeUpdateConfirmErr,
    DbUser,
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
        DbPasswordChangeUpdateConfirmErr::TokenNotFound => {
            PasswordChangeUpdateConfirmErr::TokenNotFound
        }
        DbPasswordChangeUpdateConfirmErr::Db(_) => PasswordChangeUpdateConfirmErr::InternalServer,
    }
}

fn status_code(
    result: &Result<PasswordChangeUpdateConfirmRes, PasswordChangeUpdateConfirmErr>,
) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PasswordChangeUpdateConfirmErr::NewPasswordInvalid(_)) => StatusCode::BAD_REQUEST,
        Err(PasswordChangeUpdateConfirmErr::TokenNotFound) => StatusCode::BAD_REQUEST,
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
    let time = app.get_time_micro();
    let inner =
        async || -> Result<PasswordChangeUpdateConfirmRes, PasswordChangeUpdateConfirmErr> {
            validate_password(&req.new_password)
                .map_err(|err| PasswordChangeUpdateConfirmErr::NewPasswordInvalid(err))?;
            let new_password = hash_password(&req.new_password)
                .map_err(|_err| PasswordChangeUpdateConfirmErr::InternalServer)?;

            let user_email = app
                .db
                .password_change_update_confirm(time, req.token, new_password)
                .await
                .map_err(from_db_password_change_confirm_err)?;

            let address = app.get_address().await;

            if let Some(db_user) = &*db_user {
                let email_body = send_email_password_change_confirm(address);
                let _ = app
                    .db
                    .email_sent_add(
                        time,
                        catsquad_db::DbEmailSentReason::UserPasswordChangeConfirm,
                        user_email.clone(),
                        email_body,
                    )
                    .await;
            } else {
                let email_body = send_email_password_reset_confirm(address);
                let _ = app
                    .db
                    .email_sent_add(
                        time,
                        catsquad_db::DbEmailSentReason::UserPasswordResetConfirm,
                        user_email.clone(),
                        email_body,
                    )
                    .await;
            };

            Ok(PasswordChangeUpdateConfirmRes { email: user_email })
        };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(any(test, feature = "test_server"))]
mod test_utils {
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn password_change_confirm(
            &self,
            password_change_token: Uuid,
            new_password: impl Into<String>,
            session_token: Uuid,
        ) -> Result<cs::PasswordChangeUpdateConfirmRes, cs::PasswordChangeUpdateConfirmErr>
        {
            self.client
                .password_change_update_confirm(password_change_token, new_password)
                .header_add(
                    header::COOKIE,
                    create_auth_cookie_str(uuid_to_str(session_token)),
                )
                .send()
                .await
                .into_json()
                .await
        }

        pub async fn password_change_get_latest_key(&self) -> Uuid {
            self.state.db.password_change_get_all().await.unwrap()[0].token
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_user_password_change_confirm() {
    init_log();
    let server = crate::TestServer::new(0, "test_api_user_password_change_confirm").await;

    let (user, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "hello1111111@1P")
        .await;
    let (user2, session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "hello1111111@1P")
        .await;

    let result = server.user_get_by_session_key(session_key.clone()).await;
    assert!(result.is_ok());

    // password change
    let result = server
        .password_change_add("hey@heyadora.com", session_key)
        .await
        .unwrap();

    let pss_key = server.password_change_get_latest_key().await;

    let result = server
        .password_change_confirm(pss_key.clone(), "invalid", session_key.clone())
        .await;
    assert!(matches!(
        result,
        Err(PasswordChangeUpdateConfirmErr::NewPasswordInvalid(_))
    ));

    let result = server
        .password_change_confirm(pss_key, "hello1111111@2P", session_key.clone())
        .await
        .unwrap();

    let db_user = server.state.db.user_get_by_username("hey").await.unwrap();
    verify_password("hello1111111@2P", db_user.password).unwrap();

    let emails = server
        .email_sent_get_filtered(DbEmailSentReason::UserPasswordChangeConfirm)
        .await;

    assert_eq!(emails.len(), 1);
    assert_eq!(
        emails[0].reason,
        DbEmailSentReason::UserPasswordChangeConfirm.to_string()
    );

    // password change should invalidate old sessions
    let result = server.user_get_by_session_key(session_key.clone()).await;
    assert!(result.is_err());

    // password reset

    server.state.set_time(1);

    let result = server
        .password_change_add("hey@heyadora.com", 0_u128.to_be_bytes())
        .await
        .unwrap();

    let pss_key = server.password_change_get_latest_key().await;

    let result = server
        .password_change_confirm(pss_key, "hello1111111@3P", session_key.clone())
        .await
        .unwrap();

    let db_user = server.state.db.user_get_by_username("hey").await.unwrap();
    let result = verify_password("hello1111111@3P", db_user.password);
    assert!(result.is_ok());

    let emails = server
        .email_sent_get_filtered(DbEmailSentReason::UserPasswordResetConfirm)
        .await;

    assert_eq!(emails.len(), 1);
    assert_eq!(
        emails[0].reason,
        DbEmailSentReason::UserPasswordResetConfirm.to_string()
    );

    let result = server.user_get_by_session_key(session_key.clone()).await;
    assert!(result.is_err());

    let result = server.user_get_by_session_key(session_key2.clone()).await;
    assert!(result.is_ok());
}
