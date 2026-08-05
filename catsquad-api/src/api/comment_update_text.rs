use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbCommentUpdateTextErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    CommentRes, CommentUpdateTextErr, CommentUpdateTextReq, MAX_POST_COMMENT_LENGTH,
    PostUpdateTagsErr, validate_comment_text,
};

use crate::{
    api::{comment_add::from_db_comment, post_add::from_db_post},
    state::AppState,
    utils::rng_str,
};

fn from_db_comment_update_text_err(value: DbCommentUpdateTextErr) -> CommentUpdateTextErr {
    match value {
        DbCommentUpdateTextErr::NotFound => CommentUpdateTextErr::PostNotFound,
        DbCommentUpdateTextErr::Unauthorized => {
            CommentUpdateTextErr::Unauthorized("unauthorized".to_string())
        }
        DbCommentUpdateTextErr::Db(_) => CommentUpdateTextErr::InternalServer,
    }
}

fn status_code(result: &Result<CommentRes, CommentUpdateTextErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(CommentUpdateTextErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(CommentUpdateTextErr::InvalidText(_)) => StatusCode::BAD_REQUEST,
        Err(CommentUpdateTextErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(CommentUpdateTextErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn comment_update_text(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<CommentUpdateTextReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;

    let inner = async || -> Result<CommentRes, CommentUpdateTextErr> {
        let user_id = db_user.id.clone();
        let comment_key = req.comment_key;
        let text = req.text.trim();

        validate_comment_text(text).map_err(|err| CommentUpdateTextErr::InvalidText(err))?;

        let result = app
            .db
            .comment_update_text(time, user_id, comment_key, text)
            .await
            .map_err(from_db_comment_update_text_err)?;

        Ok(from_db_comment(result))
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
        pub async fn comment_update_text(
            &self,
            comment_key: impl Into<String>,
            text: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::CommentRes, cs::CommentUpdateTextErr> {
            self.client
                .comment_update_text(comment_key, text)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_res()
                .await
        }
    }
}

#[tokio::test]
async fn test_comment_update_text() {
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
        .post_add("title", "description1", "tags1", session_key1.clone())
        .await
        .unwrap();

    let comment1 = server
        .comment_add(
            post1.key.clone(),
            None::<String>,
            "text1",
            session_key1.clone(),
        )
        .await
        .unwrap();

    assert_eq!(comment1.text, "text1");

    let comment1 = server
        .comment_update_text(comment1.key.clone(), "text2", &session_key1)
        .await
        .unwrap();

    assert_eq!(comment1.text, "text2");

    let result = server
        .comment_update_text("invalid", "text3", &session_key1)
        .await;
    assert_eq!(result, Err(CommentUpdateTextErr::PostNotFound));

    let text_invalid = rng_str(MAX_POST_COMMENT_LENGTH + 1);
    let result = server
        .comment_update_text("invalid", text_invalid, &session_key1)
        .await;
    assert!(matches!(result, Err(CommentUpdateTextErr::InvalidText(_))));

    let result = server
        .comment_update_text("invalid", "", &session_key1)
        .await;
    assert!(matches!(result, Err(CommentUpdateTextErr::InvalidText(_))));

    let result = server
        .comment_update_text(comment1.key.clone(), "text4", &session_key2)
        .await;
    assert!(matches!(result, Err(CommentUpdateTextErr::Unauthorized(_))));
}
