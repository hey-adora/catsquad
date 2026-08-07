use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateFileRemoveErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PostFile, PostRes, PostState, PostUpdateFileRemoveErr, PostUpdateFileRemoveReq,
};

use crate::{
    api::post_add::{from_db_post, from_db_post_file},
    state::AppState,
};

fn from_db_post_update_file_remove_err(
    value: DbPostUpdateFileRemoveErr,
) -> PostUpdateFileRemoveErr {
    match value {
        DbPostUpdateFileRemoveErr::PostNotFound => PostUpdateFileRemoveErr::PostNotFound,
        DbPostUpdateFileRemoveErr::Unauthorized => {
            PostUpdateFileRemoveErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateFileRemoveErr::FileNotFound => PostUpdateFileRemoveErr::FileNotFound,
        DbPostUpdateFileRemoveErr::Db(_) => PostUpdateFileRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<PostFile, PostUpdateFileRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateFileRemoveErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateFileRemoveErr::FileNotFound) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateFileRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_file_remove(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateFileRemoveReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;

    let inner = async || -> Result<PostFile, PostUpdateFileRemoveErr> {
        let user_id = db_user.id.clone();
        let post_key = req.post_key;
        let hash = req.hash;

        let post_file = app
            .db
            .post_update_file_remove(time, user_id.clone(), post_key.clone(), hash)
            .await
            .map_err(from_db_post_update_file_remove_err)?;

        // let mut post = None;
        // for file_hash in hashes {
        //     let result = app
        //         .db
        //         .post_update_file_remove(time, user_id.clone(), post_key.clone(), file_hash)
        //         .await
        //         .map_err(from_db_post_update_file_remove_err)?;
        //     post = Some(result);
        // }
        // let post = post.ok_or_else(|| PostUpdateFileRemoveErr::InternalServer)?;

        Ok(from_db_post_file(post_file))
        // Ok(from_db_post(post))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
#[tokio::test]
async fn test_post_update_file_remove() {
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
        .into_res()
        .await
        .unwrap();

    let _result = server
        .client
        .post_update_file_add(post1.key.clone(), vec!["../assets/favicon.ico".to_string()])
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    server
        .post_update_state(post1.key.clone(), PostState::Active, &session_key1)
        .await
        .unwrap();

    let result = server
        .post_get_by_key(post1.key.clone(), &session_key1)
        .await
        .unwrap();

    assert_eq!(result.file.len(), 1);
    let file1_hash = result.file[0].hash.clone();

    let _result = server
        .client
        .post_update_file_remove(post1.key.clone(), file1_hash)
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    let result = server
        .post_get_by_key(post1.key.clone(), &session_key1)
        .await
        .unwrap();

    assert_eq!(result.file.len(), 0);
}
