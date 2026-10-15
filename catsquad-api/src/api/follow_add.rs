use crate::state::AppState;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbFollow, DbFollowAddErr, DbUser};
use catsquad_shared::{FollowAddErr, FollowAddParams, FollowRes};

pub fn from_db_following(value: DbFollow) -> FollowRes {
    FollowRes {
        id: value.id,
        follower_user_username: value.follower_user_username,
        following_user_username: value.following_user_username,
        modified_at: value.modified_at,
        created_at: value.created_at,
    }
}

fn from_db_follow_add_err(value: DbFollowAddErr) -> FollowAddErr {
    match value {
        DbFollowAddErr::UserAlreadyFollowed => FollowAddErr::UserAlreadyFollowed,
        DbFollowAddErr::CantFollowYourself => FollowAddErr::CantFollowYourself,
        DbFollowAddErr::UserNotFound(_) => FollowAddErr::InternalServer,
        DbFollowAddErr::Db(_) => FollowAddErr::InternalServer,
    }
}

fn status_code(result: &Result<FollowRes, FollowAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(FollowAddErr::UserAlreadyFollowed) => StatusCode::BAD_REQUEST,
        Err(FollowAddErr::CantFollowYourself) => StatusCode::BAD_REQUEST,
        Err(FollowAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(FollowAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn follow_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Path(params): Path<FollowAddParams>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let follower_user_username = db_user.username.clone();
    let following_user_username = params.user_username;

    let inner = async || -> Result<FollowRes, FollowAddErr> {
        let follow = app
            .db
            .follow_add(time, follower_user_username, following_user_username)
            .await
            .map_err(from_db_follow_add_err)?;

        Ok(from_db_following(follow))
    };

    let result = inner().await;
    let status_code = status_code(&result);
    (status_code, Json(result))
}

#[cfg(any(test, feature = "test_server"))]
mod test_utils {
    use std::fmt::Display;

    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn follow_add(
            &self,
            following_user_username: impl Display,
            session_token: Uuid,
        ) -> Result<cs::FollowRes, cs::FollowAddErr> {
            self.client
                .follow_add(following_user_username)
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
async fn test_api_follow_add() {
    use catsquad_log::prelude::*;
    init_log();

    let server = crate::TestServer::new(0, "test_api_follow_add").await;

    let (user1, session_key1) = server
        .user_add_full("hey1", "hey1@heyadora.com", "1nnerogGeron@@$")
        .await;

    let (user2, _session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "1nnerogGeron@@$")
        .await;

    // assert error
    {
        let result = server
            .follow_add(user1.username.clone(), session_key1)
            .await;
        assert!(matches!(result, Err(FollowAddErr::CantFollowYourself)));

        let result = server
            .follow_add(user1.username.clone(), 0_u128.to_be_bytes())
            .await;
        assert!(matches!(result, Err(FollowAddErr::Unauthorized(_))));
    }

    // assert success
    {
        let _result = server
            .follow_add(user2.username.clone(), session_key1)
            .await
            .unwrap();
    }

    // assert error
    {
        let result = server
            .follow_add(user2.username.clone(), session_key1)
            .await;
        assert!(matches!(result, Err(FollowAddErr::UserAlreadyFollowed)));
    }
}
