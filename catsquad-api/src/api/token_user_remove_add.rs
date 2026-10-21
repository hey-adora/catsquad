use crate::state::AppState;
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbToken, DbTokenAddErr, DbTokenLabel, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    TokenUserRemoveAddErr, TokenUserRemoveRes, Uuid,
    link_absolute_settings_account_deletion_confirm,
    link_absolute_settings_password_change_confirm, uuid_to_str,
};
use url::Url;

fn from_db_token_user_remove(value: DbToken) -> TokenUserRemoveRes {
    TokenUserRemoveRes {
        expires: value.expires_at,
    }
}

fn from_db_token_add_err(value: DbTokenAddErr) -> TokenUserRemoveAddErr {
    match value {
        DbTokenAddErr::UserNotFound(_) => TokenUserRemoveAddErr::InternalServer,
        DbTokenAddErr::Db(_) => TokenUserRemoveAddErr::InternalServer,
    }
}

fn status_code(result: &Result<TokenUserRemoveRes, TokenUserRemoveAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(TokenUserRemoveAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(TokenUserRemoveAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_user_remove(address: Url, token: Uuid) -> String {
    let token = uuid_to_str(token);
    let link = link_absolute_settings_account_deletion_confirm(address, token)
        .unwrap()
        .to_string();
    debug!("EMAIL SENT {link}");
    link
}

pub async fn token_user_remove_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let user_remove_expiration = app.get_user_remove_expiration_micro().await;
    let expires = time + user_remove_expiration;
    let username = db_user.username.clone();
    let email = db_user.email.clone();
    let address = app.get_address().await;

    let inner = async || -> Result<TokenUserRemoveRes, TokenUserRemoveAddErr> {
        let token = app
            .db
            .token_add(time, username, expires, DbTokenLabel::AccountDeletion)
            .await
            .map_err(from_db_token_add_err)?;

        let email_body = send_email_user_remove(address, token.id);
        let _ = app
            .db
            .email_sent_add(
                time,
                catsquad_db::DbEmailSentReason::UserRemove,
                email,
                email_body,
            )
            .await;

        Ok(from_db_token_user_remove(token))
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
        pub async fn token_user_remove_add(
            &self,
            session_token: Uuid,
        ) -> Result<cs::TokenUserRemoveRes, cs::TokenUserRemoveAddErr> {
            self.client
                .token_user_remove_add()
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
async fn test_api_token_user_remove_add() {
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new(0, "test_api_token_user_remove_add").await;

    let (_user1, user_token) = server
        .user_add_full("hey", "hey@heyadora.com", "a1234567890111GG11$")
        .await;

    // assert error
    {
        let result = server.token_user_remove_add(0_u128.to_be_bytes()).await;
        assert!(matches!(
            result,
            Err(TokenUserRemoveAddErr::Unauthorized(_))
        ));
    }

    // assert success
    {
        use catsquad_db::DbEmailSentReason;

        let tokens = server.state.db.token_get_all().await.unwrap();
        assert_eq!(tokens.len(), 0);

        server.state.set_user_remove_expiration(10).await;
        server.token_user_remove_add(user_token).await.unwrap();

        let tokens = server.state.db.token_get_all().await.unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].label, DbTokenLabel::AccountDeletion);

        let emails = server
            .email_sent_get_filtered(DbEmailSentReason::UserRemove)
            .await;
        assert_eq!(emails.len(), 1);
    }
}
