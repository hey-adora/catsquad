use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbCommentRemoveErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{CommentRemoveErr, CommentRemoveReq, CommentRes};

use crate::{
    api::{comment_add::from_db_comment, post_add::from_db_post},
    state::AppState,
    utils::rng_str,
};

fn from_db_comment_remove(value: DbCommentRemoveErr) -> CommentRemoveErr {
    match value {
        DbCommentRemoveErr::CommentNotFound(_) => CommentRemoveErr::CommentNotFound,
        DbCommentRemoveErr::Unauthorized => {
            CommentRemoveErr::Unauthorized("unauthorized".to_string())
        }
        DbCommentRemoveErr::Db(_) => CommentRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<(), CommentRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(CommentRemoveErr::CommentNotFound) => StatusCode::NOT_FOUND,
        Err(CommentRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(CommentRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn comment_remove(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<CommentRemoveReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), CommentRemoveErr> {
        let user_username = db_user.username.clone();
        let comment_id = req.comment_id;

        app.db
            .comment_remove(time, user_username, comment_id)
            .await
            .map_err(from_db_comment_remove)?;

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
        pub async fn comment_remove(
            &self,
            comment_id: i64,
            session_token: Uuid,
        ) -> Result<(), cs::CommentRemoveErr> {
            self.client
                .comment_remove(comment_id)
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
async fn test_comment_remove() {
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

    server.state.set_time(0);
    let comment1 = server
        .comment_add(post1.id, 0, "text1", session_key1)
        .await
        .unwrap();

    server.state.set_time(1);
    let comment2 = server
        .comment_add(post1.id, 0, "text2", session_key1.clone())
        .await
        .unwrap();
    server.state.set_time(2);
    let comment3 = server
        .comment_add(post1.id, comment2.id.clone(), "text3", session_key1.clone())
        .await
        .unwrap();
    server.state.set_time(3);

    let comments = server.state.db.comment_get_all().await.unwrap();
    assert_eq!(comments[0].text, "text3");
    assert_eq!(comments[1].text, "text2");
    assert_eq!(comments[2].text, "text1");

    server
        .comment_remove(comment1.id.clone(), session_key1)
        .await
        .unwrap();

    let comments = server.state.db.comment_get_all().await.unwrap();
    assert_eq!(comments[0].text, "text3");
    assert_eq!(comments[1].text, "text2");

    let result = server
        .comment_remove(comment2.id.clone(), session_key2)
        .await;
    assert!(matches!(result, Err(CommentRemoveErr::Unauthorized(_))));

    let result = server.comment_remove(0, session_key2).await;
    assert!(matches!(result, Err(CommentRemoveErr::CommentNotFound)));
}
