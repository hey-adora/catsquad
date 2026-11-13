use crate::{auth::SessionTokenV2, state::AppState};
use axum::{Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbUserUpdateState, DbUserUpdateStateErr};
use catsquad_log::prelude::*;
use catsquad_shared::{UserUpdateStateErr, UserUpdateStateReq, UserUpdateStateRes};
use std::fmt::Display;

fn from_db_user_update_state(value: DbUserUpdateState) -> UserUpdateStateRes {
    UserUpdateStateRes {
        username: value.username.clone(),
    }
}

fn from_db_user_update_state_err(value: DbUserUpdateStateErr) -> UserUpdateStateErr {
    match value {
        // DbUserUpdateStateErr::UserNotFound => UserUpdateStateErr::UserNotFound,
        DbUserUpdateStateErr::Unauthorized(err) => UserUpdateStateErr::Unauthorized(err),
        DbUserUpdateStateErr::Db(_) => UserUpdateStateErr::InternalServer,
    }
}

fn status_code(result: &Result<Vec<UserUpdateStateRes>, UserUpdateStateErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserUpdateStateErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        // Err(UserUpdateStateErr::UserNotFound) => StatusCode::BAD_REQUEST,
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
    Form(params): Form<UserUpdateStateReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let usernames = params.usernames;
    let state = params.state;

    let inner = async || -> Result<Vec<UserUpdateStateRes>, UserUpdateStateErr> {
        let email_send_fn = {
            let state = state.clone();
            async move |data: DbUserUpdateState| {
                let to_email = data.email.clone();
                let username = data.username.clone();
                send_email_user_update_state(username, state)
            }
        };

        let users_data = app
            .db
            .user_update_state(
                email_send_fn,
                time,
                auth_session_token.into(),
                usernames,
                state,
            )
            .await
            .map_err(from_db_user_update_state_err)?
            .into_iter()
            .map(from_db_user_update_state)
            .collect();

        Ok(users_data)
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, UserState, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn user_update_state(
            &self,
            usernames: Vec<String>,
            new_state: UserState,
            session_token: Uuid,
        ) -> Result<Vec<cs::UserUpdateStateRes>, cs::UserUpdateStateErr> {
            self.client
                .user_update_state(usernames, new_state)
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
async fn test_api_user_update_state() {
    use crate::promote_admin;
    use catsquad_shared::UserState;

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_update_state").await;

    let pss = "a1234567890111GG11$";
    let (_user1, token1) = server.user_add_full("hey", "hey@heyadora.com", pss).await;
    let (_user2, token2) = server.user_add_full("hey2", "hey2@heyadora.com", pss).await;
    let (_user3, _token3) = server.user_add_full("hey3", "hey3@heyadora.com", pss).await;

    // assert error unauthorized
    {
        let result = server
            .user_update_state(vec!["hey2".to_string()], UserState::Suspended, token1)
            .await;
        assert!(matches!(result, Err(UserUpdateStateErr::Unauthorized(_))));

        let result = server
            .user_update_state(vec!["hey4".to_string()], UserState::Suspended, token1)
            .await;
        assert!(matches!(result, Err(UserUpdateStateErr::Unauthorized(_))));
    }

    // assert not found
    {
        server.state.set_admin_username("hey").await;
        promote_admin(server.state.clone()).await.unwrap();

        let result = server
            .user_update_state(vec!["hey4".to_string()], UserState::Suspended, token1)
            .await
            .unwrap();
        assert_eq!(result.len(), 0);
    }

    // assert error unauthorized
    {
        let result = server
            .user_update_state(vec!["hey4".to_string()], UserState::Suspended, token2)
            .await;
        assert!(matches!(result, Err(UserUpdateStateErr::Unauthorized(_))));
    }

    // assert success
    {
        let mut result = server
            .user_update_state(
                vec!["hey2".to_string(), "hey3".to_string()],
                UserState::Suspended,
                token1,
            )
            .await
            .unwrap();

        result.sort();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].username, "hey2");
        assert_eq!(result[1].username, "hey3");
    }
}
