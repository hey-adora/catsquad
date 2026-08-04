use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateTitleErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostRes, PostUpdateTitleErr, PostUpdateTitleReq, validate_post_title};

use crate::{api::post_add::from_db_post, auth::verify_password, state::AppState};

fn from_db_post_update_title_err(value: DbPostUpdateTitleErr) -> PostUpdateTitleErr {
    match value {
        DbPostUpdateTitleErr::PostNotFound => PostUpdateTitleErr::PostNotFound,
        DbPostUpdateTitleErr::Unauthorized => {
            PostUpdateTitleErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateTitleErr::UserNotFound => PostUpdateTitleErr::InternalServer,
        DbPostUpdateTitleErr::Db(_) => PostUpdateTitleErr::InternalServer,
    }
}

fn status_code(result: &Result<PostRes, PostUpdateTitleErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateTitleErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateTitleErr::InvalidTitle(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateTitleErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateTitleErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_title(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateTitleReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;

    let inner = async || -> Result<PostRes, PostUpdateTitleErr> {
        let user_id = db_user.id.clone();
        let post_key = req.post_key;
        let new_title = req.new_title;

        validate_post_title(&new_title).map_err(|err| PostUpdateTitleErr::InvalidTitle(err))?;

        let result = app
            .db
            .post_update_title(time, user_id, post_key, &new_title)
            .await
            .map_err(from_db_post_update_title_err)?;

        Ok(from_db_post(result))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[tokio::test]
async fn test_post_update_title() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new().await;

    let (user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (user2, session_key2) = server
        .user_add_full("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
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
        .post_update_title(post1.key.clone(), "title2")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    assert_eq!(post1.title, "title2");

    let result = server
        .client
        .post_update_title(post1.key.clone(), "title3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key2.clone()))
        .send()
        .await
        .into_res()
        .await;
    assert!(matches!(result, Err(PostUpdateTitleErr::Unauthorized(_))));

    let result = server
        .client
        .post_update_title("invalid", "title3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await;
    assert!(matches!(result, Err(PostUpdateTitleErr::PostNotFound)));

    let result = server
        .client
        .post_update_title("invalid", "title3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await;
    assert!(matches!(result, Err(PostUpdateTitleErr::PostNotFound)));
}
