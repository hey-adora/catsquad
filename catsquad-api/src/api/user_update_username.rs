use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbUser, DbUserUpdateUsernameErr};
use catsquad_log::prelude::*;
use catsquad_shared::{
    UserUpdateUsernameErr, UserUpdateUsernameReq, UserUpdateUsernameRes, validate_username,
};

use crate::{auth::verify_password, state::AppState};

fn from_db_user_update_username_err(value: DbUserUpdateUsernameErr) -> UserUpdateUsernameErr {
    match value {
        DbUserUpdateUsernameErr::UsernameAlreadyUsed => UserUpdateUsernameErr::UsernameAlreadyUsed,
        DbUserUpdateUsernameErr::Db(_) => UserUpdateUsernameErr::InternalServer,
    }
}

fn from_db_user_update_username(value: String) -> UserUpdateUsernameRes {
    UserUpdateUsernameRes { username: value }
}

fn status_code(result: &Result<UserUpdateUsernameRes, UserUpdateUsernameErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserUpdateUsernameErr::UsernameAlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(UserUpdateUsernameErr::InvalidUsername(_)) => StatusCode::BAD_REQUEST,
        Err(UserUpdateUsernameErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserUpdateUsernameErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(UserUpdateUsernameErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_user_update_username(
    address: impl AsRef<str>,
    new_username: impl AsRef<str>,
) -> String {
    // let link = link_absolute_reg_finish(address, token);
    let link = "placeholder change".to_string();
    debug!("EMAIL SENT {link}");
    link
}

pub async fn user_update_username(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<UserUpdateUsernameReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;

    let inner = async || -> Result<UserUpdateUsernameRes, UserUpdateUsernameErr> {
        // let email = req.email.trim().to_lowercase();
        let user_id = db_user.id.clone();
        let user_email = db_user.email.clone();
        let user_hash = db_user.password.clone();

        let password = req.password.clone();
        let new_username = req.new_username.trim().to_lowercase();

        verify_password(password, user_hash)
            .map_err(|_| UserUpdateUsernameErr::Unauthorized("invalid password".to_string()))?;

        validate_username(&new_username)
            .map_err(|err| UserUpdateUsernameErr::InvalidUsername(err))?;

        let result = app
            .db
            .user_update_username(time, user_id, &new_username)
            .await
            .map_err(from_db_user_update_username_err)?;

        let address = app.get_address().await;
        let email_body = send_email_user_update_username(address, new_username.clone());
        let _ = app
            .db
            .email_sent_add(
                0,
                catsquad_db::DbEmailSentReason::UserUsernameChange,
                user_email,
                email_body,
            )
            .await;

        Ok(from_db_user_update_username(result))
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
        pub async fn user_update_username(
            &self,
            password: impl Into<String>,
            new_username: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::UserUpdateUsernameRes, cs::UserUpdateUsernameErr> {
            self.client
                .user_update_username(password, new_username)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_res()
                .await
        }
    }
}

#[tokio::test]
async fn test_user_update_username() {
    init_log();
    let server = crate::TestServer::new().await;

    let pss = "a1234567890111GG11$";
    let (_user1, token) = server.user_add_full("hey", "hey@heyadora.com", pss).await;
    let _ = server.user_add_full("hey2", "hey2@heyadora.com", pss).await;

    let result = server.user_update_username("", "one", &token).await;
    assert!(matches!(
        result,
        Err(UserUpdateUsernameErr::Unauthorized(_))
    ));

    let result = server.user_update_username("hey2", "one", &token).await;
    assert!(matches!(
        result,
        Err(UserUpdateUsernameErr::Unauthorized(_))
    ));

    let result = server.user_update_username(pss, "he", &token).await;
    assert!(matches!(
        result,
        Err(UserUpdateUsernameErr::InvalidUsername(_))
    ));

    let result = server.user_update_username(pss, "hey2", &token).await;
    assert!(matches!(
        result,
        Err(UserUpdateUsernameErr::UsernameAlreadyUsed)
    ));

    let result = server
        .user_update_username(pss, "hey3", &token)
        .await
        .unwrap();
    assert_eq!(result.username, "hey3");
}
