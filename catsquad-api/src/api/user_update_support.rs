use crate::state::AppState;
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbUser, DbUserUpdateSupportErr};
use catsquad_shared::{UserUpdateSupportErr, UserUpdateSupportReq};

fn from_db_user_update_support_err(value: DbUserUpdateSupportErr) -> UserUpdateSupportErr {
    match value {
        DbUserUpdateSupportErr::UserNotFound => UserUpdateSupportErr::InternalServer,
        DbUserUpdateSupportErr::Db(_) => UserUpdateSupportErr::InternalServer,
    }
}

fn status_code(result: &Result<(), UserUpdateSupportErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserUpdateSupportErr::Invalid(_)) => StatusCode::BAD_REQUEST,
        Err(UserUpdateSupportErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserUpdateSupportErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_update_support(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<UserUpdateSupportReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), UserUpdateSupportErr> {
        let new_support = req.new_support;
        let user_username = db_user.username.clone();

        app.db
            .user_update_support(time, user_username, new_support)
            .await
            .map_err(from_db_user_update_support_err)?;

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
        pub async fn user_update_support(
            &self,
            new_supoprt: impl Into<String>,
            session_token: Uuid,
        ) -> Result<(), cs::UserUpdateSupportErr> {
            self.client
                .user_update_support(new_supoprt)
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
async fn test_api_user_update_support() {
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_update_support").await;

    let pss = "a1234567890111GG11$";
    let (_user1, token) = server.user_add_full("hey", "hey@heyadora.com", pss).await;
    let _ = server.user_add_full("hey2", "hey2@heyadora.com", pss).await;

    let result = server
        .user_update_support("woo3", 0_u128.to_be_bytes())
        .await;
    assert!(matches!(result, Err(UserUpdateSupportErr::Unauthorized(_))));

    let user = server
        .state
        .db
        .user_get_by_email("hey@heyadora.com")
        .await
        .unwrap();

    assert_eq!(user.aboutme, "");
    assert_eq!(user.support, "");

    server.user_update_support("woo3", token).await.unwrap();

    let user = server
        .state
        .db
        .user_get_by_email("hey@heyadora.com")
        .await
        .unwrap();

    assert_eq!(user.aboutme, "");
    assert_eq!(user.support, "woo3");
}
