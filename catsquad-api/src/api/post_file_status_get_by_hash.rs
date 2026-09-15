use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{
        StatusCode,
        header::{self},
    },
    response::IntoResponse,
};
use catsquad_db::{DbFileImage, DbFileImageGetByHashErr, DbPostImageGetByHashErr, DbUser};
use catsquad_shared::{
    PostFileGetStatusByHashParams, PostFileGetStatusByHashRes, PostFileStatusGetByHashErr,
};

use crate::state::AppState;

fn from_post_image(value: DbFileImage) -> PostFileGetStatusByHashRes {
    PostFileGetStatusByHashRes {
        is_proccesed: value.processed,
    }
}

fn from_post_image_get_by_hash_err(value: DbFileImageGetByHashErr) -> PostFileStatusGetByHashErr {
    match value {
        DbFileImageGetByHashErr::NotFound => PostFileStatusGetByHashErr::FileNotFound,
        DbFileImageGetByHashErr::Db(_) => PostFileStatusGetByHashErr::InternalServerErr,
    }
}

fn status_code(
    result: &Result<PostFileGetStatusByHashRes, PostFileStatusGetByHashErr>,
) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostFileStatusGetByHashErr::FileNotFound) => StatusCode::NOT_FOUND,
        // Err(PostFileStatusGetByHashErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostFileStatusGetByHashErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_file_status_get_by_hash(
    // db_user: Extension<Option<DbUser>>,
    State(app): State<AppState>,
    Path(params): Path<PostFileGetStatusByHashParams>,
) -> impl IntoResponse {
    // let user_username = db_user
    //     .as_ref()
    //     .map(|v| v.username.clone())
    //     .unwrap_or_default();
    // let post_id = params.post_id;
    let file_hash = params.file_hash;

    let inner = async || -> Result<PostFileGetStatusByHashRes, PostFileStatusGetByHashErr> {
        let image = app
            .db
            .file_image_get_by_hash(file_hash)
            .await
            .map_err(from_post_image_get_by_hash_err)?;

        Ok(from_post_image(image))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::TestServer;
    use catsquad_shared::{self as cs};

    impl TestServer {
        pub async fn post_file_status_get_by_hash(
            &self,
            file_hash: i64,
        ) -> Result<cs::PostFileGetStatusByHashRes, cs::PostFileStatusGetByHashErr> {
            self.client
                .post_file_status_get_by_hash(file_hash)
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_post_file_status_by_hash() {
    use catsquad_log::init_log;

    init_log();

    let server = crate::TestServer::new(0, "test_api_post_file_status_by_hash").await;

    let (_user1, session_token1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_token1)
        .await
        .unwrap();

    let result = server
        .post_update_file_add(post1.id, &["../assets/favicon.ico"], session_token1)
        .await
        .unwrap();

    let file_hash = result[0].hash;

    let result = server
        .post_file_status_get_by_hash(file_hash)
        .await
        .unwrap();

    assert!(!result.is_proccesed);
}
