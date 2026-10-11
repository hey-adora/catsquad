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
    FileImageGetStatusByHashParams, FileImageGetStatusByHashRes, FileImageStatusGetByHashErr,
};

use crate::state::AppState;

fn from_file_image(value: DbFileImage) -> FileImageGetStatusByHashRes {
    FileImageGetStatusByHashRes {
        is_proccesed: value.processed,
    }
}

fn from_file_image_get_by_hash_err(value: DbFileImageGetByHashErr) -> FileImageStatusGetByHashErr {
    match value {
        DbFileImageGetByHashErr::NotFound => FileImageStatusGetByHashErr::FileNotFound,
        DbFileImageGetByHashErr::Db(_) => FileImageStatusGetByHashErr::InternalServerErr,
    }
}

fn status_code(
    result: &Result<FileImageGetStatusByHashRes, FileImageStatusGetByHashErr>,
) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(FileImageStatusGetByHashErr::FileNotFound) => StatusCode::NOT_FOUND,
        Err(FileImageStatusGetByHashErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn file_image_status_get_by_hash(
    State(app): State<AppState>,
    Path(params): Path<FileImageGetStatusByHashParams>,
) -> impl IntoResponse {
    let file_hash = params.file_hash;

    let inner = async || -> Result<FileImageGetStatusByHashRes, FileImageStatusGetByHashErr> {
        let image = app
            .db
            .file_image_get_by_hash(file_hash)
            .await
            .map_err(from_file_image_get_by_hash_err)?;

        Ok(from_file_image(image))
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
        pub async fn file_image_status_get_by_hash(
            &self,
            file_hash: i64,
        ) -> Result<cs::FileImageGetStatusByHashRes, cs::FileImageStatusGetByHashErr> {
            self.client
                .file_image_status_get_by_hash(file_hash)
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_file_image_status_by_hash() {
    use catsquad_log::init_log;

    init_log();

    let server = crate::TestServer::new(0, "test_api_file_image_status_by_hash").await;

    let (_user1, session_token1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_token1)
        .await
        .unwrap();

    let image = server
        .post_update_image_add(post1.id, &["../assets/favicon.ico"], session_token1)
        .await
        .unwrap();

    let file_hash = image.hash;

    let result = server
        .file_image_status_get_by_hash(file_hash)
        .await
        .unwrap();

    assert!(!result.is_proccesed);
}
