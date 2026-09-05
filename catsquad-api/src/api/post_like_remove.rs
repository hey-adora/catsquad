use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostLikeRemoveErr, DbUser};
use catsquad_shared::{PostLikeRemoveErr, PostLikeRemoveReq, PostLikeRes};

use crate::{api::post_like_add::from_db_post_like, state::AppState};

fn from_db_post_like_remove_err(value: DbPostLikeRemoveErr) -> PostLikeRemoveErr {
    match value {
        DbPostLikeRemoveErr::LikeNotFound => PostLikeRemoveErr::LikeNotFound,
        DbPostLikeRemoveErr::PostNotFound => PostLikeRemoveErr::PostNotFound,
        DbPostLikeRemoveErr::Unauthorized => {
            PostLikeRemoveErr::Unauthorized("unauthorized".to_string())
        }
        DbPostLikeRemoveErr::Db(_) => PostLikeRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostLikeRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostLikeRemoveErr::LikeNotFound) => StatusCode::NOT_FOUND,
        Err(PostLikeRemoveErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostLikeRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostLikeRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_like_remove(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostLikeRemoveReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<(), PostLikeRemoveErr> {
        let post_id = req.post_id;
        let user_username = db_user.username.clone();

        app.db
            .post_like_remove(time, user_username, post_id)
            .await
            .map_err(from_db_post_like_remove_err)?;

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
        pub async fn post_like_remove(
            &self,
            post_id: i64,
            session_token: Uuid,
        ) -> Result<(), cs::PostLikeRemoveErr> {
            self.client
                .post_like_remove(post_id)
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
async fn test_post_like_remove() {
    use catsquad_log::prelude::*;
    use catsquad_shared as cs;
    init_log();
    let server = crate::TestServer::new(0, "test_post_like_remove").await;

    let (user1, session_key1) = server
        .user_add_full("hey", "hey@heyadora.com", "1nnerogGeron@@$")
        .await;
    let (user2, session_key2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "1nnerogGeron@@$")
        .await;

    let post1 = server
        .post_add("title1", "description1", "tags1", session_key1)
        .await
        .unwrap();

    {
        let result = server.post_like_remove(post1.id, session_key1).await;
        assert!(matches!(result, Err(PostLikeRemoveErr::Unauthorized(_))));
    }

    server
        .post_update_state(post1.id, cs::PostState::Active, session_key1)
        .await
        .unwrap();

    {
        server.post_like_add(post1.id, session_key2).await.unwrap();

        let result = server.post_like_remove(post1.id, session_key1).await;
        assert!(matches!(result, Err(PostLikeRemoveErr::LikeNotFound)));

        let result = server.post_like_remove(0, session_key1).await;
        assert!(matches!(result, Err(PostLikeRemoveErr::PostNotFound)));

        let result = server.post_like_remove(post1.id, session_key2).await;
        assert!(matches!(result, Ok(_)));

        let result = server.post_like_remove(post1.id, session_key2).await;
        assert!(matches!(result, Err(PostLikeRemoveErr::LikeNotFound)));
    }

    server
        .post_update_state(post1.id, cs::PostState::Hidden, session_key1)
        .await
        .unwrap();

    {
        let result = server.post_like_remove(post1.id, session_key1).await;
        assert!(matches!(result, Err(PostLikeRemoveErr::Unauthorized(_))));
    }
}
