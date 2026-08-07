use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostLike, DbPostLikeAddErr, DbUser, id_to_string};
use catsquad_log::prelude::*;
use catsquad_shared::{PostLikeAddErr, PostLikeAddReq, PostLikeRes};

use crate::{
    api::user_add::{from_db_user_redacted, from_db_user_sensitive},
    state::AppState,
};

pub fn from_db_post_like(value: DbPostLike) -> PostLikeRes {
    PostLikeRes {
        key: id_to_string(value.id),
    }
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
    let time = app.get_time().await;
    let inner = async || -> Result<PostLikeRes, PostLikeAddErr> {
        let post_key = req.post_key;
        let user_id = db_user.id.clone();

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
    use catsquad_shared as cs;

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn post_like_add(
            &self,
            post_key: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::PostLikeRes, cs::PostLikeAddErr> {
            self.client
                .post_like_add(post_key.into())
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_res()
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
        .post_add("title1", "description1", "tags1", &session_key)
        .await
        .unwrap();

    let result = server.post_like_add(post1.key.clone(), &session_key).await;
    assert!(matches!(result, Err(PostLikeAddErr::CantLikeYourself)));
}
