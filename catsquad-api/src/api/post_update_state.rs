use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateStateErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostUpdateStateErr, PostUpdateStateReq};

use crate::{api::post_add::from_db_post, state::AppState};

fn from_db_post_update_state_err(value: DbPostUpdateStateErr) -> PostUpdateStateErr {
    match value {
        DbPostUpdateStateErr::SameState => PostUpdateStateErr::SameState,
        // DbPostUpdateStateErr::PostNotActive => PostUpdateStateErr::PostNotActive,
        DbPostUpdateStateErr::CantSetDraft => PostUpdateStateErr::CantSetDraft,
        DbPostUpdateStateErr::PostNotFound => PostUpdateStateErr::PostNotFound,
        DbPostUpdateStateErr::Unauthorized => {
            PostUpdateStateErr::Unauthorized("unauthorized".to_string())
        }
        // DbPostUpdateStateErr::UserNotFound => PostUpdateStateErr::InternalServer,
        DbPostUpdateStateErr::Db(_) => PostUpdateStateErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostUpdateStateErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateStateErr::SameState) => StatusCode::BAD_REQUEST,
        Err(PostUpdateStateErr::CantSetDraft) => StatusCode::BAD_REQUEST,
        Err(PostUpdateStateErr::PostNotActive) => StatusCode::BAD_REQUEST,
        Err(PostUpdateStateErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateStateErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateStateErr::UserNotFound) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(PostUpdateStateErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_state(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateStateReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), PostUpdateStateErr> {
        let user_username = db_user.username.clone();
        let post_id = req.post_id;
        let new_state = req.new_state;

        app.db
            .post_update_state(time, user_username, post_id, new_state)
            .await
            .map_err(from_db_post_update_state_err)?;

        Ok(())
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, PostState, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn post_update_state(
            &self,
            post_id: i64,
            new_state: PostState,
            session_token: Uuid,
        ) -> Result<(), cs::PostUpdateStateErr> {
            self.client
                .post_update_state(post_id, new_state)
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
async fn test_api_post_update_state() {
    // TODO test all errors
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;
    use catsquad_shared::PostState;

    init_log();

    let server = crate::TestServer::new(0, "test_api_post_update_state").await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (_user2, session_key2) = server
        .user_add_full("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1.clone())
        .await
        .unwrap();

    server
        .post_update_state(post1.id, PostState::Active, session_key1.clone())
        .await
        .unwrap();
    let post1 = server
        .post_get_by_key(post1.id, session_key1)
        .await
        .unwrap();

    assert_eq!(post1.state, PostState::Active);
}
