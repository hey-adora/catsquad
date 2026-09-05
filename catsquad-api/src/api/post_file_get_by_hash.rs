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
use catsquad_shared::{PostFileGetByHashErr, StorageParams};
use tokio::fs;

use crate::state::AppState;

fn from_post_image_get_by_hash_err(value: DbPostImageGetByHashErr) -> PostFileGetByHashErr {
    match value {
        DbPostImageGetByHashErr::NotFound => PostFileGetByHashErr::FileNotFound,
        DbPostImageGetByHashErr::Unauthorized => {
            PostFileGetByHashErr::Unauthorized("unauthorized".to_string())
        }
        DbPostImageGetByHashErr::Db(_) => PostFileGetByHashErr::InternalServerErr,
    }
}

fn from_io_err(_value: std::io::Error) -> PostFileGetByHashErr {
    PostFileGetByHashErr::InternalServerErr
}

fn status_code(result: &Result<Vec<u8>, PostFileGetByHashErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        // Err(PostFileGetByHashErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostFileGetByHashErr::FileNotFound) => StatusCode::NOT_FOUND,
        Err(PostFileGetByHashErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostFileGetByHashErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_file_get_by_hash(
    db_user: Extension<Option<DbUser>>,
    State(app): State<AppState>,
    Path(params): Path<StorageParams>,
) -> impl IntoResponse {
    //TODO optimize this nonsense, simplify db query
    //the string manipulation at the end is -100000% performance

    let user_username = db_user
        .as_ref()
        .map(|v| v.username.clone())
        .unwrap_or_default();
    let post_id = params.post_id;
    let file_hash = params.file_hash;

    let inner = async || -> Result<(Vec<u8>, String), PostFileGetByHashErr> {
        // let post = app
        //     .db
        //     .post_get_by_id(user_usename, post_key)
        //     .await
        //     .map_err(from_post_get_by_key_err)?;

        // let file_hash = post
        //     .images_hashes
        //     .into_iter()
        //     .find(|v| *v == file_hash)
        //     .ok_or(PostFileGetByHashErr::FileNotFound)?;
        // let file_hash_str = (file_hash as u64).to_string();

        let image = app
            .db
            .post_image_get_by_hash(user_username, post_id, file_hash)
            .await
            .map_err(from_post_image_get_by_hash_err)?;
        // TODO GET FROM FILES_IMAGES TABLE

        // let file_extension = file.extension;
        let file_extension = image.extension;
        let hash_str = image.hash.to_string();
        let path = app.get_storage_path().await;
        let path = std::path::Path::new(&path);
        let path = path.join(hash_str).with_extension(&file_extension);

        let bytes = fs::read(&path)
            .await
            .inspect_err(|err| error!("{path:?} {err}"))
            .map_err(from_io_err)?;
        Ok((bytes, file_extension))
    };

    // let storage_path = app.get_storage_path().await;
    // let storage_path = std::path::Path::new(&storage_path);
    // let file_key = params.file_hash;
    // let path = storage_path.join(file_key);

    // if file_key.
    // path.

    // let bytes = fs::read(path).await;
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
    // application/json
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn post_file_get_by_hash(
            &self,
            post_id: i64,
            file_hash: i64,
            session_token: Uuid,
        ) -> Result<Vec<u8>, cs::PostFileGetByHashErr> {
            self.client
                .post_file_get_by_hash(post_id, file_hash)
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
async fn test_api_post_file_by_hash() {
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
        .post_file_get_by_hash(post1.id, file_hash, session_key1)
        .await;
    assert!(matches!(result, Err(PostFileGetByHashErr::FileNotFound)));

    server
        .post_update_state(post1.id, PostState::Active, session_key1)
        .await
        .unwrap();

    let result = server
        .post_file_get_by_hash(post1.id, file_hash, session_key1)
        .await;
    assert!(matches!(result, Err(PostFileGetByHashErr::FileNotFound)));

    let result = server
        .post_update_file_add(post1.id, &["../assets/favicon.ico"], session_key1)
        .await
        .unwrap();

    // let file_hash = result.file[0].hash.clone();

    let result = server
        .post_file_get_by_hash(post1.id, file_hash, session_key1)
        .await
        .unwrap();

    // result.
}
