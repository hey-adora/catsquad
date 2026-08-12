use axum::{
    Json,
    extract::{Query, RawPathParams, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use catsquad_db::{DbPost, DbPostSearchErr};
use catsquad_shared::{Order, PostRes, PostSearchErr, PostSearchParams, PostState, TimeRange};

use crate::{api::post_add::from_db_post, state::AppState};

pub fn from_db_posts(value: Vec<DbPost>) -> Vec<PostRes> {
    value.into_iter().map(|v| from_db_post(v)).collect()
}

fn from_db_post_search_err(value: DbPostSearchErr) -> PostSearchErr {
    match value {
        DbPostSearchErr::Db(_) => PostSearchErr::InternalServer,
    }
}

// fn params_req(value: RawPathParams) -> Result<PostGetByKeyParams, PostGetByKeyErr> {
//     value
//         .iter()
//         .find(|(name, _)| *name == POST_GET_BY_KEY_REQ_FIELD_POST_KEY)
//         .ok_or(PostGetByKeyErr::BadRequest(
//             "missing post_key param".to_string(),
//         ))
//         .map(|(_, value)| PostGetByKeyParams {
//             post_key: value.to_string(),
//         })
// }

pub fn status_code(result: &Result<Vec<PostRes>, PostSearchErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostSearchErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_search(
    State(app): State<AppState>,
    Query(req): Query<PostSearchParams>,
) -> impl IntoResponse {
    let tags = req.tags.unwrap_or_default();
    let username = req.username.unwrap_or_default();
    let time = req
        .time
        .map(|v| u128::from_str_radix(&v, 10).unwrap_or_default())
        .unwrap_or_default();
    let limit = req.limit.unwrap_or(50);
    let range = req.range.unwrap_or(TimeRange::MoreOrEqual);
    let order = req.order.unwrap_or(Order::ThreeTwoOne);
    let inner = async || -> Result<Vec<PostRes>, PostSearchErr> {
        let posts = app
            .db
            .post_search(PostState::Active, tags, username, time, limit, range, order)
            .await
            .map_err(from_db_post_search_err)?;

        Ok(from_db_posts(posts))
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
        pub async fn post_search(
            &self,
            tags: impl Into<String>,
            username: impl Into<String>,
            time: u128,
            limit: usize,
            range: TimeRange,
            order: Order,
        ) -> Result<Vec<cs::PostRes>, cs::PostSearchErr> {
            self.client
                .post_search(tags, username, time, limit, range, order)
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_search() {
    use catsquad_log::prelude::*;
    use catsquad_shared::{Order, TimeRange};

    init_log();
    let server = crate::TestServer::new().await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "a1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1.clone())
        .await
        .unwrap();

    let posts = server
        .post_search("", "", 0, 10, TimeRange::MoreOrEqual, Order::ThreeTwoOne)
        .await
        .unwrap();

    assert_eq!(posts.len(), 0);

    server
        .post_update_state(post1.key.clone(), PostState::Active, session_key1.clone())
        .await
        .unwrap();

    let posts = server
        .post_search("", "", 0, 10, TimeRange::MoreOrEqual, Order::ThreeTwoOne)
        .await
        .unwrap();

    assert_eq!(posts.len(), 1);

    // server.sea

    // let result = server.post_get_by_key("invalid").await;

    // assert_eq!(result, Err(PostGetByKeyErr::PostNotFound));

    // let result = server.post_get_by_key(post1.key.clone()).await.unwrap();

    // assert_eq!(result.key, post1.key);
}
