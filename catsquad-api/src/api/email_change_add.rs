use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{
    DbEmailChange, DbEmailChangeAddErr, DbEmailChangeToken, DbEmailSentReason, DbUser, id_to_string,
};
use catsquad_log::prelude::*;
use catsquad_shared::{EmailChangeAddErr, EmailChangeRes, EmailChangeToken};

use crate::state::AppState;

fn from_db_email_change_add_err(value: DbEmailChangeAddErr) -> EmailChangeAddErr {
    match value {
        DbEmailChangeAddErr::UserNotFound => EmailChangeAddErr::InternalServer,
        DbEmailChangeAddErr::Db(_) => EmailChangeAddErr::InternalServer,
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

pub fn status_code(result: &Result<EmailChangeRes, EmailChangeAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(EmailChangeAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(EmailChangeAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
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
    let inner = async || -> Result<EmailChangeRes, EmailChangeAddErr> {
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

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared as cs;

    impl TestServer {
        pub async fn email_change_add(
            &self,
            session_key: impl Into<String>,
        ) -> Result<cs::EmailChangeRes, cs::EmailChangeAddErr> {
            self.client
                .email_change_add()
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_res()
                .await
        }

        pub async fn email_change_get_current_token(
            &self,
            email_change_key: impl Into<String>,
        ) -> String {
            self.state
                .db
                .email_change_get_by_key(email_change_key.into())
                .await
                .unwrap()
                .current
                .token
        }

        pub async fn email_change_get_new_token(
            &self,
            email_change_key: impl Into<String>,
        ) -> String {
            self.state
                .db
                .email_change_get_by_key(email_change_key.into())
                .await
                .unwrap()
                .new
                .unwrap()
                .token
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_add() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;
    init_log();
    let server = crate::TestServer::new().await;

    let (user1, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "1g234567890111GG11$")
        .await;

    let result = server.email_change_add("invalid").await;
    assert!(matches!(result, Err(EmailChangeAddErr::Unauthorized(_))));

    let _email_change = server.email_change_add(session_key).await.unwrap();

    let emails = server
        .email_sent_get_filtered(DbEmailSentReason::UserEmailChangeAddCurrent)
        .await;
    assert_eq!(emails.len(), 1);
    assert_eq!(
        emails[0].reason,
        DbEmailSentReason::UserEmailChangeAddCurrent.to_string()
    );
}
