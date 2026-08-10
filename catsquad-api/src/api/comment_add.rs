use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbComment, DbCommentAddErr, DbUser, id_to_string};
use catsquad_shared::{CommentAddErr, CommentAddReq, CommentRes, validate_comment_text};

use crate::{
    api::user_add::{from_db_user_redacted, from_db_user_sensitive},
    state::AppState,
};

pub fn from_db_comment(value: DbComment) -> CommentRes {
    CommentRes {
        key: id_to_string(value.id),
        user: from_db_user_redacted(value.user),
        post_key: id_to_string(value.post),
        parent_key: value.parent.into_iter().map(|v| id_to_string(v)).collect(),
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
    let time = app.get_time().await;
    let inner = async || -> Result<CommentRes, CommentAddErr> {
        let post_key = req.post_key;
        let comment_key = if req.comment_key.is_empty() {
            None
        } else {
            Some(req.comment_key)
        };
        let text = req.text.trim();
        let user_id = db_user.id.clone();

        validate_comment_text(text).map_err(|err| CommentAddErr::InvalidText(err.to_string()))?;

        let comment = app
            .db
            .comment_add(time, user_id, post_key, comment_key, text)
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
    use catsquad_shared as cs;

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn comment_add(
            &self,
            post_key: impl Into<String>,
            comment_parent_key: impl Into<String>,
            text: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::CommentRes, cs::CommentAddErr> {
            self.client
                .comment_add(post_key, comment_parent_key, text)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_comment_add() {
    use catsquad_log::prelude::*;
    use catsquad_shared::MAX_POST_COMMENT_LENGTH;

    use crate::utils::rng_str;
    init_log();

    let server = crate::TestServer::new().await;

    let (user, session_key) = server
        .user_add_full("hey", "hey@heyadora.com", "1nnerogGeron@@$")
        .await;

    let post1 = server
        .post_add("title1", "description1", "tags1", &session_key)
        .await
        .unwrap();

    let comment1 = server
        .comment_add(post1.key.clone(), String::new(), "text", &session_key)
        .await
        .unwrap();

    let result = server
        .comment_add(post1.key.clone(), String::new(), "", &session_key)
        .await;
    assert!(matches!(result, Err(CommentAddErr::InvalidText(_))));

    let text_invalid = rng_str(MAX_POST_COMMENT_LENGTH + 1);
    let result = server
        .comment_add(post1.key.clone(), String::new(), text_invalid, &session_key)
        .await;
    assert!(matches!(result, Err(CommentAddErr::InvalidText(_))));
}
