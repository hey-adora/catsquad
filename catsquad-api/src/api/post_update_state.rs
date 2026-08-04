use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateStateErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostRes, PostState, PostUpdateStateErr, PostUpdateStateReq};

use crate::{api::post_add::from_db_post, state::AppState};

fn from_db_post_update_state_err(value: DbPostUpdateStateErr) -> PostUpdateStateErr {
    match value {
        DbPostUpdateStateErr::SameState => PostUpdateStateErr::SameState,
        DbPostUpdateStateErr::PostNotActive => PostUpdateStateErr::PostNotActive,
        DbPostUpdateStateErr::CantSetDraft => PostUpdateStateErr::CantSetDraft,
        DbPostUpdateStateErr::PostNotFound => PostUpdateStateErr::PostNotFound,
        DbPostUpdateStateErr::Unauthorized => {
            PostUpdateStateErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateStateErr::UserNotFound => PostUpdateStateErr::InternalServer,
        DbPostUpdateStateErr::Db(_) => PostUpdateStateErr::InternalServer,
    }
}

fn status_code(result: &Result<PostRes, PostUpdateStateErr>) -> StatusCode {
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
    let time = app.get_time().await;

    let inner = async || -> Result<PostRes, PostUpdateStateErr> {
        let user_id = db_user.id.clone();
        let post_key = req.post_key;
        let new_state = req.new_state;

        let result = app
            .db
            .post_update_state(time, user_id, post_key, new_state)
            .await
            .map_err(from_db_post_update_state_err)?;

        Ok(from_db_post(result))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[tokio::test]
async fn test_post_update_state() {
    // TODO test all errors
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new().await;

    let (_user1, session_key1) = server
        .user_add("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (_user2, session_key2) = server
        .user_add("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .client
        .post_add("title", "description1", "tags1")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    let post1 = server
        .client
        .post_update_state(post1.key.clone(), PostState::Active)
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    assert_eq!(post1.state, PostState::Active);
}
