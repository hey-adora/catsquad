use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbUser, DbUserUpdateAboutmeErr};
use catsquad_shared::{UserUpdateAboutmeErr, UserUpdateAboutmeReq};

use crate::{auth::verify_password, state::AppState};

fn from_db_user_update_aboutme_err(value: DbUserUpdateAboutmeErr) -> UserUpdateAboutmeErr {
    match value {
        DbUserUpdateAboutmeErr::UserNotFound => UserUpdateAboutmeErr::InternalServer,
        DbUserUpdateAboutmeErr::Db(_) => UserUpdateAboutmeErr::InternalServer,
    }
}

fn status_code(result: &Result<(), UserUpdateAboutmeErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserUpdateAboutmeErr::Invalid(_)) => StatusCode::BAD_REQUEST,
        Err(UserUpdateAboutmeErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserUpdateAboutmeErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// TODO add length checks to aboutme and support fields

pub async fn user_update_aboutme(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<UserUpdateAboutmeReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), UserUpdateAboutmeErr> {
        let new_aboutme = req.new_aboutme;
        let user_username = db_user.username.clone();

        app.db
            .user_update_aboutme(time, user_username, new_aboutme.clone())
            .await
            .map_err(from_db_user_update_aboutme_err)?;

        Ok(())
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
        pub async fn user_update_aboutme(
            &self,
            new_aboutme: impl Into<String>,
            session_token: Uuid,
        ) -> Result<(), cs::UserUpdateAboutmeErr> {
            self.client
                .user_update_aboutme(new_aboutme)
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
async fn test_api_user_update_aboutme() {
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_update_aboutme").await;

    let pss = "a1234567890111GG11$";
    let (_user1, token) = server.user_add_full("hey", "hey@heyadora.com", pss).await;
    let _ = server.user_add_full("hey2", "hey2@heyadora.com", pss).await;

    let result = server
        .user_update_aboutme("woo3", 0_u128.to_be_bytes())
        .await;
    assert!(matches!(result, Err(UserUpdateAboutmeErr::Unauthorized(_))));

    let user = server
        .state
        .db
        .user_get_by_email("hey@heyadora.com")
        .await
        .unwrap();

    assert_eq!(user.aboutme, "");

    server.user_update_aboutme("woo3", token).await.unwrap();

    let user = server
        .state
        .db
        .user_get_by_email("hey@heyadora.com")
        .await
        .unwrap();

    assert_eq!(user.aboutme, "woo3");
}
