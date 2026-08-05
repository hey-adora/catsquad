use axum::{
    Json,
    extract::{RawPathParams, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use catsquad_db::DbPostGetByKeyErr;
use catsquad_log::prelude::*;
use catsquad_shared::{
    POST_GET_BY_KEY_REQ_FIELD_POST_KEY, PostGetByKeyErr, PostGetByKeyParams, PostRes,
};

use crate::{api::post_add::from_db_post, state::AppState};

pub fn from_db_post_get_by_key_err(value: DbPostGetByKeyErr) -> PostGetByKeyErr {
    match value {
        DbPostGetByKeyErr::PostNotFound => PostGetByKeyErr::PostNotFound,
        DbPostGetByKeyErr::Db(_) => PostGetByKeyErr::InternalServerErr,
    }
}

fn params_req(value: RawPathParams) -> Result<PostGetByKeyParams, PostGetByKeyErr> {
    value
        .iter()
        .find(|(name, _)| *name == POST_GET_BY_KEY_REQ_FIELD_POST_KEY)
        .ok_or(PostGetByKeyErr::BadRequest(
            "missing post_key param".to_string(),
        ))
        .map(|(_, value)| PostGetByKeyParams {
            post_key: value.to_string(),
        })
}

pub fn status_code(result: &Result<PostRes, PostGetByKeyErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostGetByKeyErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostGetByKeyErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(PostGetByKeyErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_get_by_key(
    State(app): State<AppState>,
    params: axum::extract::RawPathParams,
) -> impl IntoResponse {
    let inner = async || -> Result<PostRes, PostGetByKeyErr> {
        let req = params_req(params)?;

        let post = app
            .db
            .post_get_by_key(req.post_key)
            .await
            .map_err(from_db_post_get_by_key_err)?;

        Ok(from_db_post(post))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::TestServer;
    use catsquad_shared as cs;

    impl TestServer {
        pub async fn post_get_by_key(
            &self,
            post_key: impl AsRef<str>,
        ) -> Result<cs::PostRes, cs::PostGetByKeyErr> {
            self.client
                .post_get_by_key(post_key)
                .send()
                .await
                .into_res()
                .await
        }
    }
}

#[tokio::test]
async fn test_post_get_by_key() {
    init_log();
    let server = crate::TestServer::new().await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1.clone())
        .await
        .unwrap();

    let result = server.post_get_by_key("invalid").await;

    assert_eq!(result, Err(PostGetByKeyErr::PostNotFound));

    let result = server.post_get_by_key(post1.key.clone()).await.unwrap();

    assert_eq!(result.key, post1.key);
}
