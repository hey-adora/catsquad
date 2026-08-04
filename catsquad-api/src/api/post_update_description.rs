use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateDescriptionErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PostRes, PostUpdateDescriptionErr, PostUpdateDescriptionReq, validate_post_description,
};

use crate::{api::post_add::from_db_post, state::AppState};

fn from_db_post_update_description_err(
    value: DbPostUpdateDescriptionErr,
) -> PostUpdateDescriptionErr {
    match value {
        DbPostUpdateDescriptionErr::PostNotFound => PostUpdateDescriptionErr::PostNotFound,
        DbPostUpdateDescriptionErr::Unauthorized => {
            PostUpdateDescriptionErr::Unauthorized("unauthorized".to_string())
        }
        // DbPostUpdateDescriptionErr::UserNotFound => PostUpdateDescriptionErr::InternalServer,
        DbPostUpdateDescriptionErr::Db(_) => PostUpdateDescriptionErr::InternalServer,
    }
}

fn status_code(result: &Result<PostRes, PostUpdateDescriptionErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateDescriptionErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateDescriptionErr::InvalidDescription(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateDescriptionErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateDescriptionErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_description(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateDescriptionReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;

    let inner = async || -> Result<PostRes, PostUpdateDescriptionErr> {
        let user_id = db_user.id.clone();
        let post_key = req.post_key;
        let new_description = req.new_description;

        validate_post_description(&new_description)
            .map_err(|err| PostUpdateDescriptionErr::InvalidDescription(err))?;

        let result = app
            .db
            .post_update_description(time, user_id, post_key, &new_description)
            .await
            .map_err(from_db_post_update_description_err)?;

        Ok(from_db_post(result))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[tokio::test]
async fn test_post_update_description() {
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
        .post_update_description(post1.key.clone(), "description2")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    assert_eq!(post1.description, "description2");

    let result = server
        .client
        .post_update_description(post1.key.clone(), "description3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key2.clone()))
        .send()
        .await
        .into_res()
        .await;
    assert!(matches!(
        result,
        Err(PostUpdateDescriptionErr::Unauthorized(_))
    ));

    let result = server
        .client
        .post_update_description("invalid", "title3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await;
    assert!(matches!(
        result,
        Err(PostUpdateDescriptionErr::PostNotFound)
    ));

    let result = server
        .client
        .post_update_description("invalid", "title3")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await;
    assert!(matches!(
        result,
        Err(PostUpdateDescriptionErr::PostNotFound)
    ));
}
