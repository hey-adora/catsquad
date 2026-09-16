use axum::{
    Extension,
    extract::{Path, State},
    http::{
        StatusCode,
        header::{self, CACHE_STATUS},
    },
    response::IntoResponse,
};
use catsquad_db::{DbPostGetByKeyErr, DbPostImageGetByHashErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    FILE_IMAGE_THUMBNAIL_EXTENSION, PostImageBytesGetByHashErr, PostImageBytesGetByHashParams,
    u128_to_str,
};
use tokio::fs;

use crate::{
    proccess_images::{storage_file_path, thumbnail_file_path},
    state::AppState,
};

fn from_post_image_get_by_hash_err(value: DbPostImageGetByHashErr) -> PostImageBytesGetByHashErr {
    match value {
        DbPostImageGetByHashErr::NotFound => PostImageBytesGetByHashErr::FileNotFound,
        DbPostImageGetByHashErr::Unauthorized => {
            PostImageBytesGetByHashErr::Unauthorized("unauthorized".to_string())
        }
        DbPostImageGetByHashErr::Db(_) => PostImageBytesGetByHashErr::InternalServerErr,
    }
}

fn from_io_err(_value: std::io::Error) -> PostImageBytesGetByHashErr {
    PostImageBytesGetByHashErr::InternalServerErr
}

fn status_code(result: &Result<Vec<u8>, PostImageBytesGetByHashErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        // Err(PostFileGetByHashErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostImageBytesGetByHashErr::FileNotFound) => StatusCode::NOT_FOUND,
        Err(PostImageBytesGetByHashErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostImageBytesGetByHashErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn get_file_image(
    app: AppState,
    user_username: impl Into<String>,
    post_id: i64,
    file_hash: i64,
    thumbnail: bool,
) -> impl IntoResponse {
    let inner = async || -> Result<(Vec<u8>, String), PostImageBytesGetByHashErr> {
        let image = app
            .db
            .post_image_get_by_hash(user_username.into(), post_id, file_hash)
            .await
            .map_err(from_post_image_get_by_hash_err)?;

        // TODO OPTIMIZE THIS BULLSH*T

        let storage_path = app.get_storage_path().await;
        let file_name = u128_to_str((image.hash as u64) as u128);

        let (file_extension, path) = if thumbnail {
            if !image.processed {
                return Err(PostImageBytesGetByHashErr::FileNotFound);
            }
            let extension = FILE_IMAGE_THUMBNAIL_EXTENSION.to_string();
            let path = thumbnail_file_path(storage_path, file_name);

            (extension, path)
        } else {
            let extension = image.extension;
            let path = storage_file_path(storage_path, file_name, &extension);
            (extension, path)
        };

        let bytes = fs::read(&path)
            .await
            .inspect_err(|err| error!("{path:?} {err}"))
            .map_err(from_io_err)?;
        Ok((bytes, file_extension))
    };

    let result = inner().await;
    match result {
        Ok((bytes, extension)) => {
            let extension = format!("image/{extension}");
            (StatusCode::OK, [(header::CONTENT_TYPE, extension)], bytes)
        }
        Err(err) => {
            let result = Err(err);
            let status_code = status_code(&result);
            let Ok(bytes) = serde_json::to_vec(&result) else {
                let bytes = format!("failed to serialize {result:#?}")
                    .as_bytes()
                    .to_vec();
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json".to_string())],
                    bytes,
                );
            };

            (
                status_code,
                [(header::CONTENT_TYPE, "application/json".to_string())],
                bytes,
            )
        }
    }
}

pub async fn post_file_bytes_get_by_hash(
    db_user: Extension<Option<DbUser>>,
    State(app): State<AppState>,
    Path(params): Path<PostImageBytesGetByHashParams>,
) -> impl IntoResponse {
    //TODO optimize this nonsense, simplify db query
    //the string manipulation at the end is -100000% performance

    let user_username = db_user
        .as_ref()
        .map(|v| v.username.clone())
        .unwrap_or_default();
    let post_id = params.post_id;
    let file_hash = params.file_hash;

    get_file_image(app, user_username, post_id, file_hash, false).await
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn post_image_bytes_get_by_hash(
            &self,
            post_id: i64,
            file_hash: i64,
            session_token: Uuid,
        ) -> Result<Vec<u8>, cs::PostImageBytesGetByHashErr> {
            self.client
                .post_file_bytes_get_by_hash(post_id, file_hash)
                .header_add(
                    header::COOKIE,
                    create_auth_cookie_str(uuid_to_str(session_token)),
                )
                .send()
                .await
                .into_bytes()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_post_image_bytes_by_hash() {
    use catsquad_shared::PostState;

    use crate::{auth::create_auth_cookie_str, get_file_hash_for_testing_by_path};

    init_log();
    let server = crate::TestServer::new(0, "test_api_post_file_by_hash").await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1.clone())
        .await
        .unwrap();

    let file_hash = get_file_hash_for_testing_by_path("../assets/favicon.ico").await;

    let result = server
        .post_image_bytes_get_by_hash(post1.id, file_hash, session_key1)
        .await;
    assert!(matches!(
        result,
        Err(PostImageBytesGetByHashErr::FileNotFound)
    ));

    server
        .post_update_state(post1.id, PostState::Active, session_key1)
        .await
        .unwrap();

    let result = server
        .post_image_bytes_get_by_hash(post1.id, file_hash, session_key1)
        .await;
    assert!(matches!(
        result,
        Err(PostImageBytesGetByHashErr::FileNotFound)
    ));

    let result = server
        .post_update_file_add(post1.id, &["../assets/favicon.ico"], session_key1)
        .await
        .unwrap();

    // let file_hash = result.file[0].hash.clone();

    let result = server
        .post_image_bytes_get_by_hash(post1.id, file_hash, session_key1)
        .await
        .unwrap();

    // result.
}
