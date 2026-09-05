use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbCommentUpdateTextErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    CommentRes, CommentUpdateTextErr, CommentUpdateTextReq, MAX_POST_COMMENT_LENGTH, PostState,
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

fn status_code(result: &Result<(), CommentUpdateTextErr>) -> StatusCode {
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
    let time = app.get_time_micro();

    let inner = async || -> Result<(), CommentUpdateTextErr> {
        let user_username = db_user.username.clone();
        let comment_id = req.comment_id;
        let text = req.text.trim();

        validate_comment_text(text).map_err(|err| CommentUpdateTextErr::InvalidText(err))?;

        let result = app
            .db
            .comment_update_text(time, user_username, comment_id, text)
            .await
            .map_err(from_db_comment_update_text_err)?;

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
        pub async fn comment_update_text(
            &self,
            comment_id: i64,
            text: impl Into<String>,
            session_token: Uuid,
        ) -> Result<(), cs::CommentUpdateTextErr> {
            self.client
                .comment_update_text(comment_id, text)
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
async fn test_api_comment_update_text() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new(0, "test_api_comment_update_text").await;

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

    server
        .post_update_state(post1.id, PostState::Active, session_key1)
        .await
        .unwrap();

    let comment1 = server
        .comment_add(post1.id, 0, "text1", session_key1.clone())
        .await
        .unwrap();

    assert_eq!(comment1.text, "text1");

    server
        .comment_update_text(comment1.id.clone(), "text2", session_key1)
        .await
        .unwrap();
    let comment1 = server
        .state
        .db
        .comment_get_by_id(comment1.id)
        .await
        .unwrap();

    assert_eq!(comment1.text, "text2");

    let result = server.comment_update_text(0, "text3", session_key1).await;
    assert_eq!(result, Err(CommentUpdateTextErr::PostNotFound));

    let text_invalid = rng_str(MAX_POST_COMMENT_LENGTH + 1);
    let result = server
        .comment_update_text(0, text_invalid, session_key1)
        .await;
    assert!(matches!(result, Err(CommentUpdateTextErr::InvalidText(_))));

    let result = server.comment_update_text(0, "", session_key1).await;
    assert!(matches!(result, Err(CommentUpdateTextErr::InvalidText(_))));

    let result = server
        .comment_update_text(comment1.id.clone(), "text4", session_key2)
        .await;
    assert!(matches!(result, Err(CommentUpdateTextErr::Unauthorized(_))));
}
