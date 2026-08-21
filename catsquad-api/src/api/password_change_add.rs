use std::fmt::Display;

use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{
    DbEmailSentReason, DbPasswordChange, DbPasswordChangeAddErr, DbUser, id_to_string,
};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PasswordChangeAddErr, PasswordChangeAddReq, PasswordChangeRes,
    link_absolute_login_password_reset_confirm, link_absolute_settings_password_change_confirm,
    validate_email,
};
use url::Url;

use crate::state::AppState;

fn from_db_password_change(value: DbPasswordChange) -> PasswordChangeRes {
    PasswordChangeRes {
        expires: value.expires,
    }
}

fn status_code(result: &Result<PasswordChangeRes, PasswordChangeAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PasswordChangeAddErr::InvalidEmail(_)) => StatusCode::BAD_REQUEST,
        Err(PasswordChangeAddErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(PasswordChangeAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_password_change(address: Url, password_change_key: impl Display) -> String {
    // let link = link_absolute_reg_finish(address, token);
    let link = link_absolute_settings_password_change_confirm(address, password_change_key)
        .unwrap()
        .to_string();
    // let link = "placeholder change".to_string();
    // debug!("EMAIL SENT {link}");
    link
}

fn send_email_password_reset(address: Url, token: impl Display) -> String {
    let link = link_absolute_login_password_reset_confirm(address, token).unwrap();
    // let link = link_absolute_reg_finish(address, token);
    // let link = "placeholder reset".to_string();
    // debug!("EMAIL SENT {link}");
    link.to_string()
}

pub async fn password_change_add(
    db_user: Extension<Option<DbUser>>,
    State(app): State<AppState>,
    Form(req): Form<PasswordChangeAddReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let password_change_expiration = app.get_password_change_expiration().await;
    let inner = async || -> Result<PasswordChangeRes, PasswordChangeAddErr> {
        let email = req.email.trim().to_lowercase();
        validate_email(&email).map_err(|err| PasswordChangeAddErr::InvalidEmail(err))?;

        let expires = time + password_change_expiration;
        let result = app.db.password_change_add(time, &email, expires).await;
        let password_change = match result {
            Ok(v) => v,
            Err(DbPasswordChangeAddErr::UserNotFound(_)) => {
                return Ok(PasswordChangeRes { expires });
            }
            Err(DbPasswordChangeAddErr::Db(_)) => {
                return Err(PasswordChangeAddErr::InternalServer);
            }
        };

        let address = app.get_address().await;

        if let Some(db_user) = &*db_user {
            let email_body =
                send_email_password_change(address, &id_to_string(password_change.id.clone()));
            let _ = app
                .db
                .email_sent_add(
                    time,
                    catsquad_db::DbEmailSentReason::UserPasswordChangeAdd,
                    email,
                    email_body,
                )
                .await;
        } else {
            let email_body =
                send_email_password_reset(address, &id_to_string(password_change.id.clone()));
            let _ = app
                .db
                .email_sent_add(
                    time,
                    catsquad_db::DbEmailSentReason::UserPasswordResetAdd,
                    email,
                    email_body,
                )
                .await;
        }

        Ok(from_db_password_change(password_change))
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
        pub async fn password_change_add(
            &self,
            email: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::PasswordChangeRes, cs::PasswordChangeAddErr> {
            self.client
                .password_change_add(email.into())
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[tokio::test]
async fn test_password_change_add() {
    init_log();
    let server = crate::TestServer::new().await;

    let email = "hey@heyadora.com";
    let (_user1, token) = server
        .user_add_full("hey", email, "a1234567890111GG11$")
        .await;

    {
        let result = server
            .password_change_add("hey2@heyadora.com", "invalid")
            .await;
        assert!(matches!(result, Ok(_)));

        let result = server
            .password_change_add("hey2@heyadora.com", &token)
            .await;
        assert!(matches!(result, Ok(_)));

        let result = server.password_change_add("invalid", &token).await;
        assert!(matches!(result, Err(PasswordChangeAddErr::InvalidEmail(_))));

        let result = server.password_change_add("invalid", "invalid").await;
        assert!(matches!(result, Err(PasswordChangeAddErr::InvalidEmail(_))));
    }

    // password change
    let _result = server.password_change_add(email, &token).await;
    let _password_change = server.state.db.password_change_get_all().await.unwrap()[0].clone();

    let emails = server
        .email_sent_get_filtered(DbEmailSentReason::UserPasswordChangeAdd)
        .await;
    assert_eq!(emails.len(), 1);
    assert_eq!(
        emails[0].reason,
        DbEmailSentReason::UserPasswordChangeAdd.to_string()
    );

    // password reset
    let _result = server.password_change_add(email, "invalid").await;
    let password_change = server.state.db.password_change_get_all().await.unwrap();
    assert_eq!(password_change.len(), 2);

    let emails = server
        .email_sent_get_filtered(DbEmailSentReason::UserPasswordResetAdd)
        .await;
    assert_eq!(emails.len(), 1);
    assert_eq!(
        emails[0].reason,
        DbEmailSentReason::UserPasswordResetAdd.to_string()
    );
}
