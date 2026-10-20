use crate::{auth::hash_password, state::AppState};
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbTokenUpdateConfirmErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    UserUpdatePasswordByTokenErr, UserUpdatePasswordByTokenReq, UserUpdatePasswordByTokenRes,
    validate_password,
};

fn from_db_token_update_confirm_err(err: DbTokenUpdateConfirmErr) -> UserUpdatePasswordByTokenErr {
    match err {
        DbTokenUpdateConfirmErr::Expired => UserUpdatePasswordByTokenErr::Expired,
        DbTokenUpdateConfirmErr::AlreadyUsed => UserUpdatePasswordByTokenErr::AlreadyUsed,
        DbTokenUpdateConfirmErr::TokenNotFound => UserUpdatePasswordByTokenErr::TokenNotFound,
        DbTokenUpdateConfirmErr::Db(_) => UserUpdatePasswordByTokenErr::InternalServer,
    }
}

fn status_code(
    result: &Result<UserUpdatePasswordByTokenRes, UserUpdatePasswordByTokenErr>,
) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserUpdatePasswordByTokenErr::NewPasswordInvalid(_)) => StatusCode::BAD_REQUEST,
        Err(UserUpdatePasswordByTokenErr::TokenNotFound) => StatusCode::BAD_REQUEST,
        Err(UserUpdatePasswordByTokenErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(UserUpdatePasswordByTokenErr::Expired) => StatusCode::BAD_REQUEST,
        Err(UserUpdatePasswordByTokenErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(UserUpdatePasswordByTokenErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
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

pub async fn user_update_password_by_token(
    db_user: Extension<Option<DbUser>>,
    State(app): State<AppState>,
    Form(req): Form<UserUpdatePasswordByTokenReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<UserUpdatePasswordByTokenRes, UserUpdatePasswordByTokenErr> {
        validate_password(&req.new_password)
            .map_err(|err| UserUpdatePasswordByTokenErr::NewPasswordInvalid(err))?;
        let new_password = hash_password(&req.new_password)
            .map_err(|_err| UserUpdatePasswordByTokenErr::InternalServer)?;

        let user_email = app
            .db
            .user_update_password_by_token(time, req.token, new_password)
            .await
            .map_err(from_db_token_update_confirm_err)?;

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

        Ok(UserUpdatePasswordByTokenRes { email: user_email })
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
        pub async fn user_update_password_by_token(
            &self,
            password_change_token: Uuid,
            new_password: impl Into<String>,
            session_token: Uuid,
        ) -> Result<cs::UserUpdatePasswordByTokenRes, cs::UserUpdatePasswordByTokenErr> {
            self.client
                .user_update_password_by_token(password_change_token, new_password)
                .header_add(
                    header::COOKIE,
                    create_auth_cookie_str(uuid_to_str(session_token)),
                )
                .send()
                .await
                .into_json()
                .await
        }

        pub async fn token_password_change_get_latest_key(&self) -> Uuid {
            self.state.db.token_get_all().await.unwrap()[0].id
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_user_update_password_by_token() {
    use crate::auth::verify_password;
    use catsquad_db::DbEmailSentReason;

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_update_password_by_token").await;

    let (user, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "hello1111111@1P")
        .await;
    let (user2, session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "hello1111111@1P")
        .await;

    let result = server.user_get_by_session_id(session_key.clone()).await;
    assert!(result.is_ok());

    // password change
    let result = server
        .token_password_change_add("hey@heyadora.com", session_key)
        .await
        .unwrap();

    let pss_key = server.token_password_change_get_latest_key().await;

    let result = server
        .user_update_password_by_token(pss_key.clone(), "invalid", session_key.clone())
        .await;
    assert!(matches!(
        result,
        Err(UserUpdatePasswordByTokenErr::NewPasswordInvalid(_))
    ));

    let result = server
        .user_update_password_by_token(pss_key, "hello1111111@2P", session_key.clone())
        .await
        .unwrap();

    let db_user = server
        .state
        .db
        .user_get_by_username("", "hey")
        .await
        .unwrap();
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
    let result = server.user_get_by_session_id(session_key.clone()).await;
    assert!(result.is_err());

    // password reset

    server.state.set_time(1);

    let result = server
        .token_password_change_add("hey@heyadora.com", 0_u128.to_be_bytes())
        .await
        .unwrap();

    let pss_key = server.token_password_change_get_latest_key().await;

    let result = server
        .user_update_password_by_token(pss_key, "hello1111111@3P", session_key.clone())
        .await
        .unwrap();

    let db_user = server
        .state
        .db
        .user_get_by_username("", "hey")
        .await
        .unwrap();
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

    let result = server.user_get_by_session_id(session_key.clone()).await;
    assert!(result.is_err());

    let result = server.user_get_by_session_id(session_key2.clone()).await;
    assert!(result.is_ok());
}
