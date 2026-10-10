use std::path::Path;

use axum::{
    Extension, Form, Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbUser, DbUserUpdatePfpErr};
use catsquad_log::prelude::*;
use catsquad_shared::{
    FileImage, FileImageAddErr, MAX_PFP_SIZE_BYTES, UserUpdatePfpErr, i64_to_str,
};
use tokio::fs;

use crate::{
    api::post_update_image_add::parse_multipart,
    auth::verify_password,
    proccess_images::{storage_file_path, thumbnail_file_path},
    state::AppState,
};

fn from_db_user_update_pfp_err(value: DbUserUpdatePfpErr) -> UserUpdatePfpErr {
    match value {
        DbUserUpdatePfpErr::IsSame => UserUpdatePfpErr::IsSame,
        DbUserUpdatePfpErr::ImageRemove(_) => UserUpdatePfpErr::InternalServer,
        DbUserUpdatePfpErr::Db(_) => UserUpdatePfpErr::InternalServer,
    }
}

fn status_code(result: &Result<FileImage, UserUpdatePfpErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserUpdatePfpErr::IsSame) => StatusCode::BAD_REQUEST,
        Err(UserUpdatePfpErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(UserUpdatePfpErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserUpdatePfpErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(UserUpdatePfpErr::Image(FileImageAddErr::ReadingResolutionErr(..))) => {
            StatusCode::BAD_REQUEST
        }
        Err(UserUpdatePfpErr::Image(FileImageAddErr::InvalidResolution { .. })) => {
            StatusCode::BAD_REQUEST
        }
        Err(UserUpdatePfpErr::Image(FileImageAddErr::IoErr(_))) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Err(UserUpdatePfpErr::Image(FileImageAddErr::StreamErr(_))) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Err(UserUpdatePfpErr::Image(FileImageAddErr::ImageTooBig { .. })) => {
            StatusCode::BAD_REQUEST
        }
        Err(UserUpdatePfpErr::Image(FileImageAddErr::ImageHasNoExtension(_))) => {
            StatusCode::BAD_REQUEST
        }
        Err(UserUpdatePfpErr::Image(FileImageAddErr::UnsupportedExtension(_))) => {
            StatusCode::BAD_REQUEST
        }
    }
}

pub async fn remove_images(storage_path: impl AsRef<Path>, file_hash: i64, file_extensoin: &str) {
    let storage_path = storage_path.as_ref();
    let image_hash_str = i64_to_str(file_hash);

    let image_thumbnail_path = thumbnail_file_path(storage_path, image_hash_str.as_str());
    if image_thumbnail_path.exists() {
        let result = fs::remove_file(image_thumbnail_path.as_path()).await;
        if let Err(err) = result {
            error!(
                "user_update_pfp remove_thumbnail hash: {} path: {:?} error {}",
                file_hash, image_thumbnail_path, err
            );
        } else {
            trace!("removed {image_thumbnail_path:?}");
        }
    }

    let image_storage_path =
        storage_file_path(storage_path, image_hash_str.as_str(), file_extensoin);
    let result = fs::remove_file(image_storage_path.as_path()).await;
    if let Err(err) = result {
        error!(
            "user_update_pfp remove_storage hash: {} path: {:?} error {}",
            file_hash, image_storage_path, err
        );
    } else {
        trace!("removed {image_storage_path:?}");
    }
}

pub async fn user_update_pfp(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    multipart: Multipart,
    // Form(req): Form<UserUpdateUsernameReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let user_username = db_user.username.clone();
    let storage_path = app.get_storage_path().await;
    let tmp_path = app.get_tmp_path().await;

    let inner = async || -> Result<FileImage, UserUpdatePfpErr> {
        let Some(image) = parse_multipart(
            multipart,
            storage_path.clone(),
            tmp_path,
            1,
            MAX_PFP_SIZE_BYTES,
            MAX_PFP_SIZE_BYTES,
            0,
        )
        .await?
        .first()
        .cloned() else {
            return Err(UserUpdatePfpErr::BadRequest(
                "no image found in the request".to_string(),
            ));
        };

        let callback_remove_img = async move |file_hash: i64, file_extensoin: &str| {
            trace!("ugaugauga");
            remove_images(storage_path.as_path(), file_hash, file_extensoin).await;
        };

        app.db
            .user_update_pfp(
                time,
                user_username.clone(),
                callback_remove_img,
                image.saved_image.size_bytes,
                image.saved_image.hash.clone(),
                image.extension.clone(),
                image.width,
                image.height,
            )
            .await
            .map_err(from_db_user_update_pfp_err)?;

        let file_image = FileImage {
            extension: image.extension,
            hash: image.saved_image.hash,
            proccesed: false,
            size_bytes: image.saved_image.size_bytes,
            width: image.width,
            height: image.height,
        };

        Ok(file_image)
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
        pub async fn user_update_pfp(
            &self,
            new_pfp_image: &[&str],
            session_token: Uuid,
        ) -> Result<cs::FileImage, cs::UserUpdatePfpErr> {
            self.client
                .user_update_pfp(new_pfp_image.to_vec())
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

#[cfg(test)]
#[tokio::test]
async fn test_api_user_update_pfp() {
    use crate::proccess_images::proccess_images_all;
    use catsquad_db::DbFileImageGetByHashErr;
    use tokio::fs;

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_update_pfp").await;

    let pss = "a1234567890111GG11$";
    let (user1, token) = server.user_add_full("hey", "hey@heyadora.com", pss).await;
    let _ = server.user_add_full("hey2", "hey2@heyadora.com", pss).await;

    assert_eq!(user1.pfp_image_hash, 0);

    let storage_path = server.state.get_storage_path().await;
    let tmp_path = server.state.get_tmp_path().await;

    let input1_img = server.create_img_input(1).await;
    let input2_img = server.create_img_input(2).await;

    trace!("input imgs\n{input1_img:#?}\n{input2_img:#?}");

    // upload pfp
    {
        server
            .user_update_pfp(&[input1_img.saved_path.to_str().unwrap()], token)
            .await
            .unwrap();

        let img1 = server
            .state
            .db
            .file_image_get_by_hash(input1_img.hash)
            .await
            .unwrap();

        let img2 = server
            .state
            .db
            .file_image_get_by_hash(input2_img.hash)
            .await;

        assert_eq!(img1.used_count, 1);
        assert!(matches!(img2, Err(DbFileImageGetByHashErr::NotFound)));

        assert!(input1_img.saved_path.exists());
        assert!(input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());

        assert!(input2_img.saved_path.exists());
        assert!(!input2_img.storage_path.exists());
        assert!(!input2_img.thumbnail_path.exists());

        proccess_images_all(0, server.state.db.clone(), storage_path.clone(), 1024)
            .await
            .unwrap();

        assert!(input1_img.thumbnail_path.exists());
    }

    // upload same pfp again
    {
        let result = server
            .user_update_pfp(&[input1_img.saved_path.to_str().unwrap()], token)
            .await;

        assert!(matches!(result, Err(UserUpdatePfpErr::IsSame)));

        let img1 = server
            .state
            .db
            .file_image_get_by_hash(input1_img.hash)
            .await
            .unwrap();

        let img2 = server
            .state
            .db
            .file_image_get_by_hash(input2_img.hash)
            .await;

        assert_eq!(img1.used_count, 1);
        assert!(matches!(img2, Err(DbFileImageGetByHashErr::NotFound)));

        assert!(input1_img.saved_path.exists());
        assert!(input1_img.storage_path.exists());
        assert!(input1_img.thumbnail_path.exists());

        assert!(input2_img.saved_path.exists());
        assert!(!input2_img.storage_path.exists());
        assert!(!input2_img.thumbnail_path.exists());
    }

    // upload different pfp
    {
        server
            .user_update_pfp(&[input2_img.saved_path.to_str().unwrap()], token)
            .await
            .unwrap();

        let img1 = server
            .state
            .db
            .file_image_get_by_hash(input1_img.hash)
            .await;

        let img2 = server
            .state
            .db
            .file_image_get_by_hash(input2_img.hash)
            .await
            .unwrap();

        assert!(matches!(img1, Err(DbFileImageGetByHashErr::NotFound)));
        assert_eq!(img2.used_count, 1);

        assert!(input1_img.saved_path.exists());
        assert!(!input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());

        assert!(input2_img.saved_path.exists());
        assert!(input2_img.storage_path.exists());
        assert!(!input2_img.thumbnail_path.exists());

        proccess_images_all(0, server.state.db.clone(), storage_path.clone(), 1024)
            .await
            .unwrap();

        assert!(input2_img.thumbnail_path.exists());
    }

    // assert file count in tmp

    let mut index = 0;
    let mut iter = fs::read_dir(tmp_path.clone()).await.unwrap();
    while let Some(_entry) = iter.next_entry().await.unwrap() {
        index += 1;
    }
    assert_eq!(index, 2);
}
