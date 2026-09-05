use crate::state::AppState;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbPostRemoveErr, DbUser};
use catsquad_shared::{PostRemoveErr, PostRemoveParams};

fn from_db_post_remove_err(value: DbPostRemoveErr) -> PostRemoveErr {
    match value {
        DbPostRemoveErr::NotFound(_) => PostRemoveErr::PostNotFound,
        DbPostRemoveErr::Unauthorized => PostRemoveErr::Unauthorized("unauthorized".to_string()),
        // DbPostRemoveErr::UserNotFound(_) => PostRemoveErr::Unauthorized("unauthorized".to_string()),
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
    let inner = async || -> Result<(), PostRemoveErr> {
        let user_username = db_user.username.clone();
        let post_id = req.post_id;

        app.db
            .post_remove(user_username, post_id)
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
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn post_remove(
            &self,
            post_id: i64,
            session_token: Uuid,
        ) -> Result<(), cs::PostRemoveErr> {
            self.client
                .post_remove(post_id)
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
async fn test_post_remove() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;
    use catsquad_log::prelude::*;
    use catsquad_shared::{PostGetByKeyErr, PostState};

    init_log();

    let server = crate::TestServer::new(0, "test_post_remove").await;

    let (user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (user2, session_key2) = server
        .user_add_full("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1)
        .await
        .unwrap();

    server
        .post_update_state(post1.id, PostState::Active, session_key1)
        .await
        .unwrap();

    let _result = server
        .post_get_by_key(post1.id, session_key1)
        .await
        .unwrap();

    let result = server.post_remove(post1.id, session_key2).await;
    assert!(matches!(result, Err(PostRemoveErr::Unauthorized(_))));

    let _result = server
        .post_get_by_key(post1.id, session_key1)
        .await
        .unwrap();

    let result = server.post_remove(post1.id, session_key1).await;
    assert!(matches!(result, Ok(_)));

    let result = server.post_get_by_key(post1.id, session_key1).await;

    assert!(matches!(result, Err(PostGetByKeyErr::PostNotFound)));

    let post2 = server
        .post_add("title", "description1", "tags1", session_key1)
        .await
        .unwrap();

    let result = server.post_remove(post2.id, session_key1).await;
    assert!(matches!(result, Err(PostRemoveErr::Unauthorized(_))));

    server
        .post_update_state(post2.id, PostState::Active, session_key1)
        .await
        .unwrap();

    let result = server.post_remove(post2.id, session_key1).await;
    assert!(matches!(result, Ok(_)));
}
