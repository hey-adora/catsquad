use crate::state::AppState;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbUser, DbUserGet, DbUserGetByUsernameErr};
use catsquad_shared::{UserGetByUsernameErr, UserGetByUsernameParams, UserGetRes};

pub fn from_db_user_get(value: DbUserGet) -> UserGetRes {
    UserGetRes {
        username: value.username,
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
    use crate::TestServer;
    use catsquad_shared::{self as cs};
    use std::fmt::Display;

    impl TestServer {
        pub async fn user_get_by_username(
            &self,
            username: impl Display,
        ) -> Result<cs::UserGetRes, cs::UserGetByUsernameErr> {
            self.client
                .user_get_by_username(username)
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
    use catsquad_log::prelude::*;
    init_log();
    let server = crate::TestServer::new(0, "test_api_user_get_by_username").await;

    let (user_add, _session_key) = server
        .user_add_full("hey", "prime@heyadora.com", "PAss$ord11111")
        .await;

    let result = server.user_get_by_username("hey").await.unwrap();
    assert_eq!(user_add.username, result.username);

    let result = server.user_get_by_username("hey1").await;
    assert_eq!(result, Err(UserGetByUsernameErr::NotFound));
}
