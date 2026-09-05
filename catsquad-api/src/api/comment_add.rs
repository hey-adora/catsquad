use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbComment, DbCommentAddErr, DbUser};
use catsquad_shared::{CommentAddErr, CommentAddReq, CommentRes, validate_comment_text};

use crate::{
    api::user_add::{from_db_user_redacted, from_db_user_sensitive},
    state::AppState,
};

pub fn from_db_comment(value: DbComment) -> CommentRes {
    CommentRes {
        id: value.id,
        user_username: value.user_username,
        post_id: value.post_id,
        parent_id: value.parents,
        replies_count: value.replies_count,
        text: value.text,
        modified_at: value.modified_at,
        created_at: value.created_at,
    }
}

fn from_db_comment_add_err(value: DbCommentAddErr) -> CommentAddErr {
    match value {
        DbCommentAddErr::ParentNotFound(err) => CommentAddErr::ReplyCommentNotFound(err),
        DbCommentAddErr::PostNotFound(err) => CommentAddErr::PostNotFound(err),
        DbCommentAddErr::UserNotFound(_) => CommentAddErr::InternalServer,
        DbCommentAddErr::Unauthorized => CommentAddErr::Unauthorized("unauthorized".to_string()),
        DbCommentAddErr::Db(_) => CommentAddErr::InternalServer,
    }
}

fn status_code(result: &Result<CommentRes, CommentAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(CommentAddErr::ReplyCommentNotFound(_)) => StatusCode::BAD_REQUEST,
        Err(CommentAddErr::PostNotFound(_)) => StatusCode::BAD_REQUEST,
        Err(CommentAddErr::InvalidText(_)) => StatusCode::BAD_REQUEST,
        Err(CommentAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(CommentAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn comment_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<CommentAddReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<CommentRes, CommentAddErr> {
        let post_id = req.post_id;
        let comment_id = if req.comment_id == 0 {
            None
        } else {
            Some(req.comment_id)
        };
        let text = req.text.trim();
        let user_username = db_user.username.clone();

        validate_comment_text(text).map_err(|err| CommentAddErr::InvalidText(err.to_string()))?;

        let comment = app
            .db
            .comment_add(time, user_username, post_id, comment_id, text)
            .await
            .map_err(from_db_comment_add_err)?;

        Ok(from_db_comment(comment))
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
        pub async fn comment_add(
            &self,
            post_id: i64,
            comment_parent_id: i64,
            text: impl Into<String>,
            session_token: Uuid,
        ) -> Result<cs::CommentRes, cs::CommentAddErr> {
            self.client
                .comment_add(post_id, comment_parent_id, text)
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
async fn test_api_comment_add() {
    use catsquad_log::prelude::*;
    use catsquad_shared::{MAX_POST_COMMENT_LENGTH, PostState};

    use crate::utils::rng_str;
    init_log();

    let server = crate::TestServer::new(0, "test_api_comment_add").await;

    let (user, session_token) = server
        .user_add_full("hey", "hey@heyadora.com", "1nnerogGeron@@$")
        .await;

    let (user2, session_token2) = server
        .user_add_full("hey2", "hey2@heyadora.com", "1nnerogGeron@@$")
        .await;

    let post1 = server
        .post_add("title1", "description1", "tags1", session_token)
        .await
        .unwrap();

    let result = server.comment_add(post1.id, 0, "text", session_token).await;
    assert!(matches!(result, Err(CommentAddErr::Unauthorized(_))));

    let result = server
        .comment_add(post1.id, 0, "text", session_token2)
        .await;
    assert!(matches!(result, Err(CommentAddErr::Unauthorized(_))));

    server
        .post_update_state(post1.id, PostState::Active, session_token)
        .await
        .unwrap();

    server
        .comment_add(post1.id, 0, "text", session_token)
        .await
        .unwrap();

    server
        .comment_add(post1.id, 0, "text1", session_token2)
        .await
        .unwrap();

    server
        .post_update_state(post1.id, PostState::Hidden, session_token)
        .await
        .unwrap();

    server
        .comment_add(post1.id, 0, "text2", session_token)
        .await
        .unwrap();

    let result = server
        .comment_add(post1.id, 0, "text3", session_token2)
        .await;
    assert!(matches!(result, Err(CommentAddErr::Unauthorized(_))));

    let result = server.comment_add(post1.id, 0, "", session_token).await;
    assert!(matches!(result, Err(CommentAddErr::InvalidText(_))));

    let text_invalid = rng_str(MAX_POST_COMMENT_LENGTH + 1);
    let result = server
        .comment_add(post1.id, 0, text_invalid, session_token)
        .await;
    assert!(matches!(result, Err(CommentAddErr::InvalidText(_))));
}
