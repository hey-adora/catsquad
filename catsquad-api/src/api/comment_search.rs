use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbComment, DbCommentSearchErr};
use catsquad_shared::{CommentRes, CommentSearchErr, CommentSearchParams};

use crate::{api::comment_add::from_db_comment, auth::SessionTokenV2, state::AppState};

fn from_db_comments(comments: Vec<DbComment>) -> Vec<CommentRes> {
    comments.into_iter().map(from_db_comment).collect()
}

fn from_db_comment_search_err(value: DbCommentSearchErr) -> CommentSearchErr {
    match value {
        DbCommentSearchErr::Unauthorized(err) => CommentSearchErr::Unauthorized(err),
        DbCommentSearchErr::Db(_) => CommentSearchErr::InternalServer,
    }
}

fn status_code(result: &Result<Vec<CommentRes>, CommentSearchErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(CommentSearchErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(CommentSearchErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn comment_search(
    auth_session_token: SessionTokenV2,
    State(app): State<AppState>,
    Query(req): Query<CommentSearchParams>,
) -> impl IntoResponse {
    // let time = app.get_time_micro();

    let inner = async || -> Result<Vec<CommentRes>, CommentSearchErr> {
        let result = app
            .db
            .comment_search(
                auth_session_token.into(),
                // time,
                req.post_id,
                if req.comment_id == 0 {
                    None
                } else {
                    Some(req.comment_id)
                },
                req.time,
                req.limit,
                req.range,
                req.order,
                req.flatten,
            )
            .await
            .map_err(from_db_comment_search_err)?;

        Ok(from_db_comments(result))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::TestServer;
    use catsquad_shared::{self as cs, Order, TimeRange};

    impl TestServer {
        pub async fn comment_search(
            &self,
            post_id: i64,
            comment_id: i64,
            time: u64,
            limit: usize,
            range: TimeRange,
            order: Order,
            flatten: bool,
        ) -> Result<Vec<cs::CommentRes>, cs::CommentSearchErr> {
            self.client
                .comment_search(post_id, comment_id, time, limit, range, order, flatten)
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_comment_search() {
    use catsquad_log::prelude::*;
    use catsquad_shared::PostState;

    init_log();

    let server = crate::TestServer::new(0, "test_api_comment_search").await;

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

    // use catsquad_shared::ToForm;

    // #[derive(serde::Serialize, Clone)]
    // struct Test1 {
    //     foo: usize,
    //     bar: Option<usize>,
    // }

    // let a = Test1 {
    //     foo: 1,
    //     bar: None::<usize>,
    // };

    // let result = a.to_form().unwrap();

    // assert_eq!(result, "");

    let comment1 = server
        .comment_add(post1.id, 0, "text1", session_key1.clone())
        .await
        .unwrap();

    let comments = server
        .comment_search(
            post1.id,
            0,
            0,
            10,
            TimeRange::None,
            catsquad_shared::Order::ThreeTwoOne,
            false,
        )
        .await
        .unwrap();

    assert_eq!(comments.len(), 1);
}
