use axum::{
    Extension,
    extract::{Path, State},
    http::{
        StatusCode,
        header::{self, CACHE_STATUS},
    },
    response::IntoResponse,
};
use catsquad_db::{DbPostFile, DbPostGetByKeyErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostFileGetByHashErr, StorageParams};
use tokio::fs;

use crate::state::AppState;

fn from_post_get_by_key_err(value: DbPostGetByKeyErr) -> PostFileGetByHashErr {
    match value {
        DbPostGetByKeyErr::PostNotFound => PostFileGetByHashErr::PostNotFound,
        DbPostGetByKeyErr::Unauthorized => {
            PostFileGetByHashErr::Unauthorized("unauthorized".to_string())
        }
        DbPostGetByKeyErr::Db(_) => PostFileGetByHashErr::InternalServerErr,
    }
}

fn from_io_err(_value: std::io::Error) -> PostFileGetByHashErr {
    PostFileGetByHashErr::InternalServerErr
}

fn status_code(result: &Result<Vec<u8>, PostFileGetByHashErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostFileGetByHashErr::PostNotFound) => StatusCode::NOT_FOUND,
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

    let user_id = db_user.as_ref().map(|v| v.id.clone());
    let post_key = params.post_key;
    let file_hash = params.file_hash;

    let inner = async || -> Result<(Vec<u8>, String), PostFileGetByHashErr> {
        let post = app
            .db
            .post_get_by_key(user_id, post_key)
            .await
            .map_err(from_post_get_by_key_err)?;

        let file = post
            .file
            .into_iter()
            .find(|v| v.hash == file_hash)
            .ok_or(PostFileGetByHashErr::FileNotFound)?;

        let file_hash = file.hash;
        let file_extension = file.extension;
        let path = app.get_storage_path().await;
        let path = std::path::Path::new(&path);
        let path = path.join(file_hash).with_extension(file_extension.clone());

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
    use catsquad_shared as cs;

    impl TestServer {
        pub async fn post_file_get_by_hash(
            &self,
            post_key: impl AsRef<str>,
            file_hash: impl AsRef<str>,
            session_key: impl AsRef<str>,
        ) -> Result<Vec<u8>, cs::PostFileGetByHashErr> {
            self.client
                .post_file_get_by_hash(post_key, file_hash)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.as_ref()))
                .send()
                .await
                .into_bytes()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_file_by_hash() {
    use catsquad_shared::PostState;

    use crate::{auth::create_auth_cookie_str, get_file_hash_for_testing_by_path};

    init_log();
    let server = crate::TestServer::new().await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1.clone())
        .await
        .unwrap();

    let file_hash = get_file_hash_for_testing_by_path("../assets/favicon.ico").await;

    let result = server
        .post_file_get_by_hash(&post1.key, &file_hash, &session_key1)
        .await;
    assert!(matches!(result, Err(PostFileGetByHashErr::PostNotFound)));

    let result = server
        .post_update_state(post1.key.clone(), PostState::Active, &session_key1)
        .await
        .unwrap();

    let result = server
        .post_file_get_by_hash(&post1.key, &file_hash, &session_key1)
        .await;
    assert!(matches!(result, Err(PostFileGetByHashErr::FileNotFound)));

    let result = server
        .client
        .post_update_file_add(post1.key.clone(), vec!["../assets/favicon.ico".to_string()])
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    // let file_hash = result.file[0].hash.clone();

    let result = server
        .post_file_get_by_hash(&post1.key, &file_hash, &session_key1)
        .await
        .unwrap();

    // result.
}
