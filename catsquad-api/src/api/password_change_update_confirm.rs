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

#[cfg(test)]
mod test_utils {
    use axum::http::header;
    use catsquad_shared as cs;

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn password_change_confirm(
            &self,
            password_change_key: impl Into<String>,
            new_password: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::PasswordChangeUpdateConfirmRes, cs::PasswordChangeUpdateConfirmErr>
        {
            self.client
                .password_change_update_confirm(password_change_key, new_password)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_password_change_confirm() {
    use catsquad_db::id_to_string;
    init_log();
    let server = crate::TestServer::new().await;

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
        .password_change_add("hey@heyadora.com", session_key.clone())
        .await
        .unwrap();

    let pss_key = id_to_string(
        server.state.db.password_change_get_all().await.unwrap()[0]
            .id
            .clone(),
    );

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

    let result = server.user_get_by_session_key(session_key.clone()).await;
    assert!(result.is_err());

    // password reset

    server.state.set_time(1).await;

    let result = server
        .password_change_add("hey@heyadora.com", "invalid")
        .await
        .unwrap();

    let pss_key = id_to_string(
        server.state.db.password_change_get_all().await.unwrap()[0]
            .id
            .clone(),
    );

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
