use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateTagsErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostRes, PostUpdateTagsErr, PostUpdateTagsReq, validate_post_tags};

use crate::{api::post_add::from_db_post, state::AppState};

fn from_db_post_update_tags_err(value: DbPostUpdateTagsErr) -> PostUpdateTagsErr {
    match value {
        DbPostUpdateTagsErr::PostNotFound => PostUpdateTagsErr::PostNotFound,
        DbPostUpdateTagsErr::Unauthorized => {
            PostUpdateTagsErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateTagsErr::UserNotFound => PostUpdateTagsErr::InternalServer,
        DbPostUpdateTagsErr::Db(_) => PostUpdateTagsErr::InternalServer,
    }
}

fn status_code(result: &Result<PostRes, PostUpdateTagsErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateTagsErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateTagsErr::InvalidTags(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateTagsErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateTagsErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_tags(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateTagsReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;

    let inner = async || -> Result<PostRes, PostUpdateTagsErr> {
        let user_id = db_user.id.clone();
        let post_key = req.post_key;
        let new_tags = req.new_tags;

        validate_post_tags(&new_tags).map_err(|err| PostUpdateTagsErr::InvalidTags(err))?;

        let result = app
            .db
            .post_update_tags(time, user_id, post_key, &new_tags)
            .await
            .map_err(from_db_post_update_tags_err)?;

        Ok(from_db_post(result))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[tokio::test]
async fn test_post_update_tags() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new().await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (_user2, session_key2) = server
        .user_add_full("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .client
        .post_add("title", "description1", "tags1")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    let post1 = server
        .client
        .post_update_tags(post1.key.clone(), "     tagS2")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    assert_eq!(post1.tags, " tags2 ");

    let result = server
        .client
        .post_update_tags(post1.key.clone(), "tags3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key2.clone()))
        .send()
        .await
        .into_json()
        .await;
    assert!(matches!(result, Err(PostUpdateTagsErr::Unauthorized(_))));

    let result = server
        .client
        .post_update_tags("invalid", "tags3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_json()
        .await;
    assert!(matches!(result, Err(PostUpdateTagsErr::PostNotFound)));

    let result = server
        .client
        .post_update_tags("invalid", "tags3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_json()
        .await;
    assert!(matches!(result, Err(PostUpdateTagsErr::PostNotFound)));
}
