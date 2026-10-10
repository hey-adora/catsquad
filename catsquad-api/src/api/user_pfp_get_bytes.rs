use crate::{proccess_images::thumbnail_file_path, state::AppState};
use axum::{
    extract::{Path, State},
    http::{
        StatusCode,
        header::{self},
    },
    response::IntoResponse,
};
use catsquad_db::DbUserGetByUsernameErr;
use catsquad_log::prelude::*;
use catsquad_shared::{
    FILE_IMAGE_THUMBNAIL_EXTENSION, UserPfpGetBytesErr, UserPfpGetBytesParams, i64_to_str,
};
use std::fmt::{Debug, Display};
use tokio::fs;

fn from_post_image_get_by_hash_err(value: DbUserGetByUsernameErr) -> UserPfpGetBytesErr {
    match value {
        DbUserGetByUsernameErr::NotFound => UserPfpGetBytesErr::UserNotFound,
        DbUserGetByUsernameErr::Db(_) => UserPfpGetBytesErr::InternalServerErr,
    }
}

fn from_io_err(value: std::io::Error) -> UserPfpGetBytesErr {
    let kind = value.kind();
    match kind {
        std::io::ErrorKind::NotFound => UserPfpGetBytesErr::StillProccesing,
        _ => UserPfpGetBytesErr::InternalServerErr,
    }
}

fn status_code(result: &Result<Vec<u8>, UserPfpGetBytesErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserPfpGetBytesErr::StillProccesing) => StatusCode::NOT_FOUND,
        Err(UserPfpGetBytesErr::PfpNotFound) => StatusCode::NOT_FOUND,
        Err(UserPfpGetBytesErr::UserNotFound) => StatusCode::NOT_FOUND,
        Err(UserPfpGetBytesErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn img_or_json_result<E, Ext>(
    result: Result<(Vec<u8>, Ext), E>,
    status: StatusCode,
) -> impl IntoResponse
where
    E: serde::Serialize + Debug,
    Ext: Display,
{
    match result {
        Ok((bytes, extension)) => {
            let extension = format!("image/{extension}");
            (StatusCode::OK, [(header::CONTENT_TYPE, extension)], bytes)
        }
        Err(err) => {
            let result = Err::<Vec<u8>, E>(err);
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
                status,
                [(header::CONTENT_TYPE, "application/json".to_string())],
                bytes,
            )
        }
    }
}

pub async fn user_pfp_get_bytes(
    // db_user: Extension<Option<DbUser>>,
    State(app): State<AppState>,
    Path(params): Path<UserPfpGetBytesParams>,
) -> impl IntoResponse {
    // TODO add check if suspended acc in the future

    let username = params.username;
    let storage_path = app.get_storage_path().await;

    let inner = async || -> Result<Vec<u8>, UserPfpGetBytesErr> {
        let user = app
            .db
            .user_get_by_username(username)
            .await
            .map_err(from_post_image_get_by_hash_err)?;

        let hash = user.pfp_image_hash;
        let hash_str = i64_to_str(hash);

        if hash == 0 {
            return Err(UserPfpGetBytesErr::PfpNotFound);
        }

        let thumbnail_path = thumbnail_file_path(storage_path, hash_str);

        let bytes = fs::read(thumbnail_path.as_path())
            .await
            .inspect_err(|err| error!("{thumbnail_path:?} {err}"))
            .map_err(from_io_err)?;

        Ok(bytes)
    };

    let result = inner().await;
    let status_code = status_code(&result);
    let result = result.map(|v| (v, FILE_IMAGE_THUMBNAIL_EXTENSION));
    img_or_json_result(result, status_code)
}

#[cfg(test)]
mod test_utils {
    use crate::TestServer;
    use catsquad_shared::{self as cs};
    use std::fmt::Display;

    impl TestServer {
        pub async fn user_pfp_get_bytes(
            &self,
            username: impl Display,
        ) -> Result<Vec<u8>, cs::UserPfpGetBytesErr> {
            self.client
                .user_pfp_get_bytes(username)
                .send()
                .await
                .into_bytes()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_user_pfp_get_bytes() {
    use crate::proccess_images::proccess_images_all;

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_pfp_get_bytes").await;
    let input1_img = server.create_img_input(0).await;
    let storage_path = server.state.get_storage_path().await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let result = server.user_pfp_get_bytes("prime").await;
    assert!(matches!(result, Err(UserPfpGetBytesErr::PfpNotFound)));

    server
        .user_update_pfp(&[input1_img.saved_path.to_str().unwrap()], session_key1)
        .await
        .unwrap();

    let result = server.user_pfp_get_bytes("prime").await;
    assert!(matches!(result, Err(UserPfpGetBytesErr::StillProccesing)));

    proccess_images_all(0, server.state.db.clone(), storage_path, 1280)
        .await
        .unwrap();

    let result = server.user_pfp_get_bytes("prime").await;
    assert!(matches!(result, Ok(_)));
}
