use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostLike, DbPostLikeAddErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostLikeAddErr, PostLikeAddReq, PostLikeRes};

use crate::{
    api::user_add::{from_db_user_redacted, from_db_user_sensitive},
    state::AppState,
};

pub fn from_db_post_like(value: DbPostLike) -> PostLikeRes {
    PostLikeRes { id: value.id }
}

// pub fn from_db_post_file(value: DbPostFile) -> PostFile {
//     PostFile {
//         extension: value.extension,
//         hash: value.hash,
//         proccesed: value.proccesed,
//         size_bytes: value.size_bytes,
//         width: value.width,
//         height: value.height,
//     }
// }

fn from_db_post_like_add_err(value: DbPostLikeAddErr) -> PostLikeAddErr {
    match value {
        DbPostLikeAddErr::CantLikeYourself => PostLikeAddErr::CantLikeYourself,
        DbPostLikeAddErr::PostWasAlreadyLiked => PostLikeAddErr::AlreadyLiked,
        DbPostLikeAddErr::PostNotFound(_) => PostLikeAddErr::PostNotFound,
        DbPostLikeAddErr::Unauthorized => PostLikeAddErr::PostNotFound,
        DbPostLikeAddErr::Db(_) => PostLikeAddErr::InternalServer,
    }
}

fn status_code(result: &Result<PostLikeRes, PostLikeAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostLikeAddErr::CantLikeYourself) => StatusCode::BAD_REQUEST,
        Err(PostLikeAddErr::AlreadyLiked) => StatusCode::BAD_REQUEST,
        Err(PostLikeAddErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostLikeAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostLikeAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_like_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostLikeAddReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<PostLikeRes, PostLikeAddErr> {
        let post_key = req.post_id;
        let user_id = db_user.username.clone();

        let post_like = app
            .db
            .post_like_add(time, user_id, post_key)
            .await
            .map_err(from_db_post_like_add_err)?;

        Ok(from_db_post_like(post_like))
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
        pub async fn post_like_add(
            &self,
            post_id: i64,
            session_token: Uuid,
        ) -> Result<cs::PostLikeRes, cs::PostLikeAddErr> {
            self.client
                .post_like_add(post_id.into())
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
async fn test_post_like_add() {
    init_log();
    let server = crate::TestServer::new().await;

    let email = "hey@heyadora.com";
    let password = "1nnerogGeron@@$";
    let (user, session_key) = server.user_add_full("hey", email, password).await;

    let post1 = server
        .post_add("title1", "description1", "tags1", session_key)
        .await
        .unwrap();

    let result = server.post_like_add(post1.id, session_key).await;
    assert!(matches!(result, Err(PostLikeAddErr::CantLikeYourself)));
}
