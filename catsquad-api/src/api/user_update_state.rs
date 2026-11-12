use std::fmt::Display;

use axum::{
    Extension, Form, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::DbUserUpdateStateErr;
use catsquad_log::prelude::*;
use catsquad_shared::{UserState, UserUpdateStateErr, UserUpdateStateParams, UserUpdateStateRes};

use crate::{
    auth::{SessionTokenV2, verify_password},
    promote_admin,
    state::AppState,
};

fn from_db_user_update_state_err(value: DbUserUpdateStateErr) -> UserUpdateStateErr {
    match value {
        DbUserUpdateStateErr::UserNotFound => UserUpdateStateErr::UserNotFound,
        DbUserUpdateStateErr::Unauthorized(err) => UserUpdateStateErr::Unauthorized(err),
        DbUserUpdateStateErr::Db(_) => UserUpdateStateErr::InternalServer,
    }
}

fn status_code(result: &Result<UserUpdateStateRes, UserUpdateStateErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserUpdateStateErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserUpdateStateErr::UserNotFound) => StatusCode::BAD_REQUEST,
        Err(UserUpdateStateErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(UserUpdateStateErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_user_update_state(username: impl Display, new_state: impl Display) -> String {
    let body = format!(
        "your account \"{}\" state was changed to \"{}\"",
        username, new_state
    );
    debug!("EMAIL SENT {body}");
    body
}

pub async fn user_update_state(
    auth_session_token: SessionTokenV2,
    State(app): State<AppState>,
    Path(params): Path<UserUpdateStateParams>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let username = params.username;
    let state = params.state;

    let inner = async || -> Result<UserUpdateStateRes, UserUpdateStateErr> {
        let email_send_fn = {
            let username = username.clone();
            let state = state.clone();
            async move || send_email_user_update_state(username, state)
        };

        app.db
            .user_update_state(
                email_send_fn,
                time,
                auth_session_token.into(),
                username,
                state,
            )
            .await
            .map_err(from_db_user_update_state_err)?;

        Ok(UserUpdateStateRes {})
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use std::fmt::Display;

    use axum::http::header;
    use catsquad_shared::{self as cs, UserState, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn user_update_state(
            &self,
            username: impl Display,
            new_state: UserState,
            session_token: Uuid,
        ) -> Result<cs::UserUpdateStateRes, cs::UserUpdateStateErr> {
            self.client
                .user_update_state(username, new_state)
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
async fn test_api_user_update_state() {
    init_log();
    let server = crate::TestServer::new(0, "test_api_user_update_state").await;

    let pss = "a1234567890111GG11$";
    let (_user1, token1) = server.user_add_full("hey", "hey@heyadora.com", pss).await;
    let (_user2, token2) = server.user_add_full("hey2", "hey2@heyadora.com", pss).await;

    // assert error
    {
        let result = server
            .user_update_state("hey2", UserState::Suspended, token1)
            .await;
        assert!(matches!(result, Err(UserUpdateStateErr::Unauthorized(_))));

        let result = server
            .user_update_state("hey4", UserState::Suspended, token1)
            .await;
        assert!(matches!(result, Err(UserUpdateStateErr::Unauthorized(_))));
    }

    // assert success
    {
        server.state.set_admin_username("hey").await;
        promote_admin(server.state.clone()).await.unwrap();

        let result = server
            .user_update_state("hey4", UserState::Suspended, token1)
            .await;
        assert!(matches!(result, Err(UserUpdateStateErr::UserNotFound)));
    }

    // let result = server.user_update_username("", "one", token).await;
    // assert!(matches!(
    //     result,
    //     Err(UserUpdateUsernameErr::Unauthorized(_))
    // ));

    // let result = server.user_update_username("hey2", "one", token).await;
    // assert!(matches!(
    //     result,
    //     Err(UserUpdateUsernameErr::Unauthorized(_))
    // ));

    // let result = server.user_update_username(pss, "he", token).await;
    // assert!(matches!(
    //     result,
    //     Err(UserUpdateUsernameErr::InvalidUsername(_))
    // ));

    // let result = server.user_update_username(pss, "hey2", token).await;
    // assert!(matches!(
    //     result,
    //     Err(UserUpdateUsernameErr::UsernameAlreadyUsed)
    // ));

    // server
    //     .user_update_username(pss, "hey3", token)
    //     .await
    //     .unwrap();
    // let user = server
    //     .state
    //     .db
    //     .user_get_by_email("hey@heyadora.com")
    //     .await
    //     .unwrap();
    // assert_eq!(user.username, "hey3");
}
