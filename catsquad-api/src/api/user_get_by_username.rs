use crate::{auth::SessionTokenV2, state::AppState};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbUser, DbUserGet, DbUserGetByUsernameErr};
use catsquad_log::prelude::*;
use catsquad_shared::{UserGetByUsernameErr, UserGetByUsernameParams, UserGetRes, UserState};

pub fn from_db_user_get(value: DbUserGet) -> UserGetRes {
    UserGetRes {
        username: value.username,
        state: UserState::try_from(value.state)
            .inspect_err(|err| error!("{err}"))
            .unwrap_or(UserState::Active),
        pfp_hash: value.pfp_image_hash,
        pfp_proccesed: value.pfp_image_proccesed,
        followers: value.followers_count,
        followed: value.followed,
        likes: value.likes_count,
        support: value.support,
        aboutme: value.aboutme,
        created_at: value.created_at,
    }
}

fn from_db_user_get_by_username(value: DbUserGetByUsernameErr) -> UserGetByUsernameErr {
    match value {
        DbUserGetByUsernameErr::NotFound => UserGetByUsernameErr::NotFound,
        DbUserGetByUsernameErr::Unauthorized(err) => UserGetByUsernameErr::Unauthorized(err),
        DbUserGetByUsernameErr::Db(_) => UserGetByUsernameErr::InternalServer,
        DbUserGetByUsernameErr::InternalServerErr => UserGetByUsernameErr::InternalServer,
    }
}

fn status_code(result: &Result<UserGetRes, UserGetByUsernameErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserGetByUsernameErr::NotFound) => StatusCode::NOT_FOUND,
        Err(UserGetByUsernameErr::Unauthorized(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(UserGetByUsernameErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_get_by_username(
    auth_session_token: SessionTokenV2,
    State(app): State<AppState>,
    Path(params): Path<UserGetByUsernameParams>,
) -> impl IntoResponse {
    let username = params.username;

    let inner = async || -> Result<UserGetRes, UserGetByUsernameErr> {
        let user = app
            .db
            .user_get_by_username(auth_session_token.into(), username)
            .await
            .map_err(from_db_user_get_by_username)?;

        Ok(from_db_user_get(user))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(any(test, feature = "test_server"))]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};
    use std::fmt::Display;

    impl TestServer {
        pub async fn user_get_by_username(
            &self,
            username: impl Display,
            session_token: Uuid,
        ) -> Result<cs::UserGetRes, cs::UserGetByUsernameErr> {
            self.client
                .user_get_by_username(username)
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
async fn test_api_user_get_by_username() {
    use crate::promote_admin;
    use catsquad_log::prelude::*;
    use catsquad_shared::UUID_ZERO;
    use catsquad_shared::{SessionTokenKind, UserState};

    init_log();

    let server = crate::TestServer::new(0, "test_api_user_get_by_username").await;

    let (user1_add, session1_key) = server
        .user_add_full("hey", "prime@heyadora.com", "PAss$ord11111")
        .await;

    let (user2_add, session2_key) = server
        .user_add_full("hey2", "prime2@heyadora.com", "PAss$ord11111")
        .await;

    // assert error
    {
        let result = server.user_get_by_username("hey1", UUID_ZERO).await;
        assert_eq!(result, Err(UserGetByUsernameErr::NotFound));
    }

    // assert success
    {
        let result = server.user_get_by_username("hey", UUID_ZERO).await.unwrap();
        assert_eq!(user1_add.username, result.username);
    }

    // assert error unauthoirized
    {
        server
            .state
            .db
            .user_update_state(
                async |_| String::new(),
                0,
                SessionTokenKind::Root,
                vec!["hey".to_string()],
                UserState::Suspended,
            )
            .await
            .unwrap();

        let result = server.user_get_by_username("hey", UUID_ZERO).await;
        assert!(matches!(result, Err(UserGetByUsernameErr::Unauthorized(_))));

        let result = server.user_get_by_username("hey", session2_key).await;
        assert!(matches!(result, Err(UserGetByUsernameErr::Unauthorized(_))));

        let result = server.user_get_by_username("hey", session1_key).await;
        assert!(matches!(result, Err(UserGetByUsernameErr::Unauthorized(_))));

        server.state.set_admin_username("hey2").await;
        promote_admin(server.state.clone()).await.unwrap();

        let result = server.user_get_by_username("hey", session1_key).await;
        assert!(matches!(result, Err(UserGetByUsernameErr::Unauthorized(_))));

        let result = server.user_get_by_username("hey", session2_key).await;
        assert!(matches!(result, Ok(_)));
    }
}
