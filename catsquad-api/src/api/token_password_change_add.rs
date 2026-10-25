use crate::state::AppState;
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbToken, DbTokenAddErr, DbTokenLabel, DbUser, DbUserGet, DbUserGetByEmailErr};
use catsquad_shared::{
    TokenPasswordChangeAddErr, TokenPasswordChangeAddReq, TokenPasswordChangeRes, Uuid,
    link_absolute_login_password_reset_confirm, link_absolute_settings_password_change_confirm,
    uuid_to_str, validate_email,
};
use url::Url;

fn from_db_token_password_change(value: DbToken) -> TokenPasswordChangeRes {
    TokenPasswordChangeRes {
        expires: value.expires_at,
    }
}

fn status_code(result: &Result<TokenPasswordChangeRes, TokenPasswordChangeAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(TokenPasswordChangeAddErr::InvalidEmail(_)) => StatusCode::BAD_REQUEST,
        Err(TokenPasswordChangeAddErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(TokenPasswordChangeAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_password_change(address: Url, token: Uuid) -> String {
    // let link = link_absolute_reg_finish(address, token);
    let token = uuid_to_str(token);
    let link = link_absolute_settings_password_change_confirm(address, token)
        .unwrap()
        .to_string();
    // let link = "placeholder change".to_string();
    // debug!("EMAIL SENT {link}");
    link
}

fn send_email_password_reset(address: Url, token: Uuid) -> String {
    let token = uuid_to_str(token);
    let link = link_absolute_login_password_reset_confirm(address, token).unwrap();
    // let link = link_absolute_reg_finish(address, token);
    // let link = "placeholder reset".to_string();
    // debug!("EMAIL SENT {link}");
    link.to_string()
}

pub async fn token_password_change_add(
    db_user: Extension<Option<DbUserGet>>,
    State(app): State<AppState>,
    Form(req): Form<TokenPasswordChangeAddReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let password_change_expiration = app.get_password_change_expiration_micro().await;
    let inner = async || -> Result<TokenPasswordChangeRes, TokenPasswordChangeAddErr> {
        let email = req.email.trim().to_lowercase();
        let expires = time + password_change_expiration;

        validate_email(&email).map_err(|err| TokenPasswordChangeAddErr::InvalidEmail(err))?;

        // TODO maybe combine both quries into one

        let result = app.db.user_get_by_email(&email).await;
        let username = match result {
            Ok(v) => v.username,
            Err(DbUserGetByEmailErr::NotFound) => {
                return Ok(TokenPasswordChangeRes { expires });
            }
            Err(DbUserGetByEmailErr::Db(_)) => {
                return Err(TokenPasswordChangeAddErr::InternalServer);
            }
        };

        let result = app
            .db
            .token_add(time, &username, expires, DbTokenLabel::PasswordChange)
            .await;
        let token = match result {
            Ok(v) => v,
            Err(DbTokenAddErr::UserNotFound(_)) => {
                return Ok(TokenPasswordChangeRes { expires });
            }
            Err(DbTokenAddErr::Db(_)) => {
                return Err(TokenPasswordChangeAddErr::InternalServer);
            }
        };

        let address = app.get_address().await;

        if let Some(db_user) = &*db_user {
            let email_body = send_email_password_change(address, token.id);
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
            let email_body = send_email_password_reset(address, token.id);
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

        Ok(from_db_token_password_change(token))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn token_password_change_add(
            &self,
            email: impl Into<String>,
            session_token: Uuid,
        ) -> Result<cs::TokenPasswordChangeRes, cs::TokenPasswordChangeAddErr> {
            self.client
                .token_password_change_add(email.into())
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

#[cfg(test)]
#[tokio::test]
async fn test_api_token_password_change_add() {
    use catsquad_db::DbEmailSentReason;
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new(0, "test_api_token_password_change_add").await;

    let email = "hey@heyadora.com";
    let (_user1, token) = server
        .user_add_full("hey", email, "a1234567890111GG11$")
        .await;

    {
        let result = server
            .token_password_change_add("hey2@heyadora.com", 0_u128.to_be_bytes())
            .await;
        assert!(matches!(result, Ok(_)));

        let result = server
            .token_password_change_add("hey2@heyadora.com", token)
            .await;
        assert!(matches!(result, Ok(_)));

        let result = server.token_password_change_add("invalid", token).await;
        assert!(matches!(
            result,
            Err(TokenPasswordChangeAddErr::InvalidEmail(_))
        ));

        let result = server
            .token_password_change_add("invalid", 0_u128.to_be_bytes())
            .await;
        assert!(matches!(
            result,
            Err(TokenPasswordChangeAddErr::InvalidEmail(_))
        ));
    }

    // password change
    let _result = server.token_password_change_add(email, token).await;
    let _token = server.state.db.token_get_all().await.unwrap()[0].clone();

    let emails = server
        .email_sent_get_filtered(DbEmailSentReason::UserPasswordChangeAdd)
        .await;
    assert_eq!(emails.len(), 1);
    assert_eq!(
        emails[0].reason,
        DbEmailSentReason::UserPasswordChangeAdd.to_string()
    );

    // password reset
    let _result = server
        .token_password_change_add(email, 0_u128.to_be_bytes())
        .await;
    let password_change = server.state.db.token_get_all().await.unwrap();
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
