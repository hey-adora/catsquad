use axum::{
    Extension, Form, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbPostLikeGetByPostErr, DbUser};
use catsquad_shared::{PostLikeGetByPostErr, PostLikeGetByPostParams};

use crate::{api::post_like_add::from_db_post_like, state::AppState};

fn status_code(result: &Result<bool, PostLikeGetByPostErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostLikeGetByPostErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_like_get_by_post(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Path(req): Path<PostLikeGetByPostParams>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<bool, PostLikeGetByPostErr> {
        let post_key = req.post_key;
        let user_id = db_user.id.clone();

        let post_like = app.db.post_like_get_by_post(time, user_id, post_key).await;

        let result = match post_like {
            Ok(_) => true,
            Err(DbPostLikeGetByPostErr::NotFound) => false,
            Err(DbPostLikeGetByPostErr::Db(_)) => return Err(PostLikeGetByPostErr::InternalServer),
        };

        Ok(result)
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
        pub async fn post_like_get_by_post(
            &self,
            post_key: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<bool, cs::PostLikeRemoveErr> {
            self.client
                .post_like_get_by_post(post_key.into())
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[tokio::test]
async fn test_post_like_get_by_post() {
    use catsquad_log::prelude::*;
    use catsquad_shared as cs;
    init_log();
    let server = crate::TestServer::new().await;

    let (user1, session_key1) = server
        .user_add_full("hey", "hey@heyadora.com", "1nnerogGeron@@$")
        .await;
    let (user2, session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "1nnerogGeron@@$")
        .await;

    let post1 = server
        .post_add("title1", "description1", "tags1", &session_key1)
        .await
        .unwrap();

    let liked = server
        .post_like_get_by_post(post1.key.clone(), session_key1.clone())
        .await
        .unwrap();

    assert!(!liked);

    server
        .post_update_state(&post1.key, cs::PostState::Active, &session_key1)
        .await
        .unwrap();

    let liked = server
        .post_like_get_by_post(post1.key.clone(), session_key1.clone())
        .await
        .unwrap();

    assert!(!liked);

    server
        .post_like_add(&post1.key, &session_key2)
        .await
        .unwrap();

    let liked = server
        .post_like_get_by_post(&post1.key, &session_key2)
        .await
        .unwrap();

    assert!(liked);
}
