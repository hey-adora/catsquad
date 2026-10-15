use crate::state::AppState;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbFollowRemoveErr, DbUser};
use catsquad_shared::{FollowRemoveErr, FollowRemoveParams};

fn from_db_follow_remove_err(value: DbFollowRemoveErr) -> FollowRemoveErr {
    match value {
        DbFollowRemoveErr::NotFound => FollowRemoveErr::NotFound,
        DbFollowRemoveErr::Db(_) => FollowRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<(), FollowRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(FollowRemoveErr::NotFound) => StatusCode::NOT_FOUND,
        Err(FollowRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(FollowRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn follow_remove(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Path(params): Path<FollowRemoveParams>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let follower_user_username = db_user.username.clone();
    let following_user_username = params.user_username;

    let inner = async || -> Result<(), FollowRemoveErr> {
        app.db
            .follow_remove(time, follower_user_username, following_user_username)
            .await
            .map_err(from_db_follow_remove_err)?;

        Ok(())
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
        pub async fn follow_remove(
            &self,
            following_user_username: impl Display,
            session_token: Uuid,
        ) -> Result<(), cs::FollowRemoveErr> {
            self.client
                .follow_remove(following_user_username)
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
async fn test_api_follow_remove() {
    use catsquad_log::prelude::*;
    init_log();

    let server = crate::TestServer::new(0, "test_api_follow_remove").await;

    let (user1, session_key1) = server
        .user_add_full("hey1", "hey1@heyadora.com", "1nnerogGeron@@$")
        .await;

    let (user2, _session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "1nnerogGeron@@$")
        .await;

    // assert error
    {
        let result = server
            .follow_remove(user2.username.clone(), session_key1)
            .await;
        assert!(matches!(result, Err(FollowRemoveErr::NotFound)));

        let result = server
            .follow_remove(user1.username.clone(), 0_u128.to_be_bytes())
            .await;
        assert!(matches!(result, Err(FollowRemoveErr::Unauthorized(_))));
    }

    // assert success
    {
        let _result = server
            .follow_add(user2.username.clone(), session_key1)
            .await
            .unwrap();

        let _result = server
            .follow_remove(user2.username.clone(), session_key1)
            .await;
    }
}
