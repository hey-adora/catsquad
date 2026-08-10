use crate::{api::post_add::from_db_post, state::AppState};
use axum::{
    Extension, Form, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbPostRemoveErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostGetByKeyErr, PostRemoveErr, PostRemoveParams, PostState};

fn from_db_post_remove_err(value: DbPostRemoveErr) -> PostRemoveErr {
    match value {
        DbPostRemoveErr::NotFound(_) => PostRemoveErr::PostNotFound,
        DbPostRemoveErr::Unauthorized => PostRemoveErr::Unauthorized("unauthorized".to_string()),
        DbPostRemoveErr::UserNotFound(_) => PostRemoveErr::Unauthorized("unauthorized".to_string()),
        DbPostRemoveErr::Db(_) => PostRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostRemoveErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_remove(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Path(req): Path<PostRemoveParams>,
) -> impl IntoResponse {
    let time = app.get_time().await;

    let inner = async || -> Result<(), PostRemoveErr> {
        let user_id = db_user.id.clone();
        let post_key = req.post_key;

        app.db
            .post_remove(user_id, post_key)
            .await
            .map_err(from_db_post_remove_err)?;

        Ok(())
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use axum::http::header;
    use catsquad_shared as cs;

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn post_remove(
            &self,
            post_key: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<(), cs::PostRemoveErr> {
            self.client
                .post_remove(post_key.into())
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[tokio::test]
async fn test_post_remove() {
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
        .into_json()
        .await
        .unwrap();

    server
        .post_update_state(&post1.key, PostState::Active, &session_key1)
        .await
        .unwrap();

    let _result = server
        .post_get_by_key(&post1.key, &session_key1)
        .await
        .unwrap();

    let result = server.post_remove(&post1.key, &session_key2).await;
    assert!(matches!(result, Err(PostRemoveErr::Unauthorized(_))));

    let _result = server
        .post_get_by_key(&post1.key, &session_key1)
        .await
        .unwrap();

    let result = server.post_remove(&post1.key, &session_key1).await;
    assert!(matches!(result, Ok(_)));

    let result = server.post_get_by_key(&post1.key, &session_key1).await;

    assert!(matches!(result, Err(PostGetByKeyErr::PostNotFound)));

    let post2 = server
        .client
        .post_add("title", "description1", "tags1")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    let result = server.post_remove(&post2.key, &session_key1).await;
    assert!(matches!(result, Ok(_)));
}
