use std::{
    hash::DefaultHasher,
    io,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use axum::{
    Extension, Form, Json,
    extract::{Multipart, RawPathParams, State},
    http::StatusCode,
    response::IntoResponse,
};
use bytes::Bytes;
use catsquad_db::{DbPostUpdateImageAddErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    POST_UPDATE_IMAGE_ADD_PARAMS_FIELD_POST_ID, PostImage, PostUpdateImageAddErr,
    PostUpdateImageAddParams, SUPPORTED_IMAGE_EXTENSIONS, u128_to_str, uuid_to_str,
};
use futures::{Stream, TryStreamExt};
use futures_util::StreamExt;
use tokio::{
    // fs::File,
    fs,
    io::{AsyncWriteExt, BufWriter},
};

use crate::{
    proccess_images::{storage_file_path, tmp_file_path},
    state::AppState,
};

fn from_db_post_update_image_add(value: DbPostUpdateImageAddErr) -> PostUpdateImageAddErr {
    match value {
        DbPostUpdateImageAddErr::OutOfStorage => PostUpdateImageAddErr::ImageTooBig {
            image_name: "uwnkown".to_string(),
            max: 0,
            got: 0,
        },
        DbPostUpdateImageAddErr::ImageTooBig => PostUpdateImageAddErr::ImageTooBig {
            image_name: "uwnkown".to_string(),
            max: 0,
            got: 0,
        },
        DbPostUpdateImageAddErr::PostNotFound => PostUpdateImageAddErr::PostNotFound,
        DbPostUpdateImageAddErr::ImageAlreadyExists => PostUpdateImageAddErr::Duplicate,
        DbPostUpdateImageAddErr::Unauthorized => {
            PostUpdateImageAddErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateImageAddErr::Db(_) => PostUpdateImageAddErr::InternalServer,
        DbPostUpdateImageAddErr::InternalError(_) => PostUpdateImageAddErr::InternalServer,
    }
}

fn status_code(result: &Result<Vec<PostImage>, PostUpdateImageAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateImageAddErr::NotImagesFound) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::Duplicate) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::ParamNotFoundPostId) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::ReadingResolutionErr(..)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::InvalidResolution { .. }) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::IoErr(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(PostUpdateImageAddErr::StreamErr(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(PostUpdateImageAddErr::ImageTooBig { .. }) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::ImageHasNoExtension(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::UnsupportedExtension(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateImageAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateImageAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn params_req(value: RawPathParams) -> Result<PostUpdateImageAddParams, PostUpdateImageAddErr> {
    value
        .iter()
        .find(|(name, _)| *name == POST_UPDATE_IMAGE_ADD_PARAMS_FIELD_POST_ID)
        .ok_or(PostUpdateImageAddErr::ParamNotFoundPostId)
        .map(|(_, value)| PostUpdateImageAddParams {
            post_id: i64::from_str_radix(value, 10).unwrap_or_default(),
        })
}

pub struct Image {
    pub saved_image: SavedImage,
    pub extension: String,
    pub width: u32,
    pub height: u32,
    // pub saved_path: PathBuf,
    // pub size_bytes: u64,
}

pub async fn parse_multipart(
    mut multipart: Multipart,
    storage_path: impl AsRef<Path>,
    tmp_path: impl AsRef<Path>,
    max_storage: u32,
    max_storage_per_image: u32,
    mut used_storage: u32,
) -> Result<Vec<Image>, PostUpdateImageAddErr> {
    let mut images = Vec::new();
    let storage_path = storage_path.as_ref();
    let tmp_path = tmp_path.as_ref();

    let mut inner = async || -> Result<(), PostUpdateImageAddErr> {
        while let Ok(Some(field)) = multipart.next_field().await {
            let image_name = if let Some(image_name) = field.file_name() {
                image_name.to_owned()
            } else {
                continue;
            };

            let Some(extension) = Path::new(&image_name).extension().and_then(|v| v.to_str())
            else {
                return Err(PostUpdateImageAddErr::ImageHasNoExtension(
                    image_name.to_string(),
                ));
            };
            let is_supported = SUPPORTED_IMAGE_EXTENSIONS
                .into_iter()
                .any(|v| *v == extension);
            if !is_supported {
                return Err(PostUpdateImageAddErr::UnsupportedExtension(
                    extension.to_string(),
                ));
            }

            let storage_left = max_storage.saturating_sub(used_storage);
            let storage_per_image = if storage_left < max_storage_per_image {
                storage_left
            } else {
                max_storage_per_image
            };

            let stream = field.map_err(io::Error::other);
            let image =
                handle_image_saving(stream, extension, storage_path, storage_per_image, tmp_path)
                    .await
                    .map_err(|err| match err {
                        SaveImageErr::ImageTooBig {
                            got_bytes,
                            max_bytes,
                        } => PostUpdateImageAddErr::ImageTooBig {
                            image_name: image_name.to_string(),
                            max: max_bytes,
                            got: got_bytes,
                        },
                        SaveImageErr::IoErr(err) => PostUpdateImageAddErr::IoErr(err.to_string()),
                        SaveImageErr::StreamErr(err) => {
                            PostUpdateImageAddErr::StreamErr(err.to_string())
                        }
                    })?;

            let result = get_img_resolution(image.saved_path.to_str().unwrap()).await;
            let (width, height) = match result {
                Ok(v) => v,
                Err(err) => {
                    tokio::fs::remove_file(&image.saved_path)
                        .await
                        .map_err(|err| PostUpdateImageAddErr::IoErr(err.to_string()))?;
                    return Err(PostUpdateImageAddErr::ReadingResolutionErr(err.to_string()));
                }
            };

            if width == 0 || height == 0 {
                tokio::fs::remove_file(&image.saved_path)
                    .await
                    .map_err(|err| PostUpdateImageAddErr::IoErr(err.to_string()))?;
                return Err(PostUpdateImageAddErr::InvalidResolution { width, height });
            }

            used_storage += image.size_bytes;

            images.push(Image {
                saved_image: image,
                extension: extension.to_string(),
                width,
                height,
            });
        }
        Ok(())
    };

    let result = inner().await;

    if let Err(err) = result {
        for image in images {
            let path = image.saved_image.saved_path;
            fs::remove_file(&path)
                .await
                .map_err(|err| PostUpdateImageAddErr::IoErr(err.to_string()))?;
        }
        return Err(err);
    };

    // result?;

    Ok(images)
}

pub struct SavedImage {
    pub hash: i64,
    pub saved_path: PathBuf,
    pub size_bytes: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum SaveImageErr {
    #[error("max image size {max_bytes} bytes, upload stopped at {got_bytes} bytes")]
    ImageTooBig { max_bytes: u32, got_bytes: u32 },

    #[error("io err {0}")]
    IoErr(#[from] std::io::Error),

    #[error(transparent)]
    StreamErr(#[from] anyhow::Error),
}

pub async fn handle_image_saving<S, StreamErr>(
    mut stream: S,
    extension: impl AsRef<str>,
    storage_path: impl AsRef<Path>,
    max_storage_per_image: u32,
    tmp_path: impl AsRef<Path>,
    // used_storage: usize,
    // max_storage: usize,
) -> Result<SavedImage, SaveImageErr>
where
    S: StreamExt + Stream<Item = Result<Bytes, StreamErr>> + Unpin,
    StreamErr: Sync + Send,
    SaveImageErr: From<StreamErr>, // S::Item: Error + Try,
{
    use rand::distr::SampleString;
    use std::hash::Hasher;
    let tmp_path = tmp_path.as_ref();
    let storage_path = storage_path.as_ref();
    let extension = extension.as_ref();
    let tmp_name = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 16);
    // tmp_name.push_str("_upload");
    let image_path_tmp = tmp_file_path(tmp_path, tmp_name, extension);
    // let file_path_tmp = tmp_path.join(&tmp_name).with_extension(extension);
    // let file_path_tmp = Path::new("/tmp/").join(&tmp_name).with_extension("part");
    // extension.as_ref()
    let image = fs::File::create(&image_path_tmp).await?;
    let mut image = BufWriter::new(image);

    let mut hasher = DefaultHasher::default();
    // let mut hasher = GxHasher::with_seed(0);
    let mut size = 0_u32;

    while let Some(value) = stream.next().await {
        let bytes = value?;
        size += bytes.len() as u32;
        if size > max_storage_per_image {
            image.flush().await?;
            drop(image);
            tokio::fs::remove_file(image_path_tmp).await?;
            return Err(SaveImageErr::ImageTooBig {
                max_bytes: max_storage_per_image,
                got_bytes: size,
            });
        }
        hasher.write(&bytes);
        image.write(&bytes).await?;
    }

    image.flush().await?;
    let hash = hasher.finish();
    let hash_str = u128_to_str(hash as u128);
    trace!("hashing in prod {image_path_tmp:?} = {hash}");

    // uuid_to_str(uuid)

    let image_path = {
        let image_path = storage_file_path(storage_path, hash_str, extension);
        // let file_path = storage_path.join(&hash_str).with_extension(extension);
        if image_path.exists() {
            trace!("image removed");
            tokio::fs::remove_file(image_path_tmp).await?;
        } else {
            trace!("image moved");
            // TODO remove file on any error
            tokio::fs::rename(&image_path_tmp, &image_path)
                .await
                .inspect_err(|err| {
                    error!(
                        "move err from {image_path_tmp:?} to {}/{} {err}",
                        std::env::current_dir().unwrap().to_str().unwrap(),
                        image_path.clone().to_str().unwrap(),
                    )
                })?;
        }
        image_path
    };

    Ok(SavedImage {
        hash: hash as i64,
        size_bytes: size,
        saved_path: image_path,
    })
}

pub async fn get_img_resolution(img_path: impl AsRef<str>) -> anyhow::Result<(u32, u32)> {
    let mut command = tokio::process::Command::new("ffprobe");
    let command = command.args(&[
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height",
        "-of",
        "csv=s=x:p=0",
        img_path.as_ref(),
    ]);
    let result = command.output().await?;
    // TODO does NOTHING
    let code = result.status.code().unwrap_or(-1);
    if code != 0 {
        return Err(anyhow!("getting resolution failed"));
    }
    let result = String::from_utf8(result.stdout)?;
    let result = result.trim();
    trace!("command output {result}");

    resolution_from_str(result)
}

pub fn resolution_from_str(res: impl AsRef<str>) -> anyhow::Result<(u32, u32)> {
    let res = res.as_ref();
    // let width = ['0'; 11];
    // let height = ['0'; 11];
    // let mut index: usize = 0;
    let x_pos = res
        .chars()
        .position(|v| v == 'x')
        .ok_or_else(|| anyhow!("x was not found, example input: 10x10, received: {res}"))?;
    if res.len() <= x_pos + 1 {
        return Err(anyhow!(
            "invalid input, example input: 10x10, received: {res}"
        ));
    }
    let input = &res[..x_pos];
    let width =
        u32::from_str_radix(input, 10).map_err(|v| anyhow!("input \"{input}\" err: {v}"))?;
    let input = &res[x_pos + 1..];
    let height =
        u32::from_str_radix(input, 10).map_err(|v| anyhow!("input \"{input}\" err: {v}"))?;

    Ok((width, height))
    // for c in res.chars() {
    //     if c >= '0' {
    //         width
    //     }
    // }
}

pub async fn post_update_image_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    // Form(req): Form<EmailChangeUpdateNewAddReq>,
    params: axum::extract::RawPathParams,
    multipart: Multipart,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let max_storage = db_user.max_storage_bytes;
    let max_storage_per_image = db_user.max_storage_per_image_bytes;
    let user_username = db_user.username.clone();
    let used_storage = db_user.used_storage_bytes;
    let storage_path = app.get_storage_path().await;
    let tmp_path = app.get_tmp_path().await;

    let inner = async || -> Result<Vec<PostImage>, PostUpdateImageAddErr> {
        let req = params_req(params)?;

        let post_id = req.post_id;

        let images = parse_multipart(
            multipart,
            storage_path,
            tmp_path,
            max_storage,
            max_storage_per_image,
            used_storage,
        )
        .await?;

        let mut post_images = Vec::new();
        // let mut post = None;
        for image in images {
            let _result = app
                .db
                .post_update_image_add(
                    time,
                    user_username.clone(),
                    post_id,
                    image.saved_image.size_bytes,
                    image.saved_image.hash.clone(),
                    image.extension.clone(),
                    image.width,
                    image.height,
                )
                .await
                .map_err(from_db_post_update_image_add)?;

            // post = Some(result);
            post_images.push(PostImage {
                extension: image.extension,
                hash: image.saved_image.hash,
                proccesed: false,
                size_bytes: image.saved_image.size_bytes,
                width: image.width,
                height: image.height,
            });
        }

        // let post = post.ok_or_else(|| PostUpdateFileAddErr::NotFilesFound)?;

        if post_images.is_empty() {
            return Err(PostUpdateImageAddErr::NotImagesFound);
        }

        Ok(post_images)
        // Ok(from_db_post(post))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use std::path::{Path, PathBuf};

    use crate::{
        TestServer, auth::create_auth_cookie_str, get_file_hash_for_testing_by_path,
        proccess_images::storage_file_path,
    };
    use axum::http::header;
    use catsquad_shared::{self as cs, PostImage, PostState, Uuid, u128_to_str, uuid_to_str};

    impl TestServer {
        pub async fn post_update_image_add(
            &self,
            // post_id: i64,
            // new_tags: impl Into<String>,
            post_id: i64,
            images: &[&str],
            session_token: Uuid,
        ) -> Result<Vec<PostImage>, cs::PostUpdateImageAddErr> {
            self.client
                .post_update_image_add(post_id, images.into_iter().map(|v| v.to_string()).collect())
                .header_add(
                    header::COOKIE,
                    create_auth_cookie_str(uuid_to_str(session_token)),
                )
                .send()
                .await
                .into_json()
                .await
            // self.client
            //     .post_update_tags(post_id, new_tags)
            //     .header_add(
            //         header::COOKIE,
            //         create_auth_cookie_str(uuid_to_str(session_token)),
            //     )
            //     .send()
            //     .await
            //     .into_json()
            //     .await
        }

        pub async fn get_image_from_storage_path(
            &self,
            storage_path: impl AsRef<Path>,
            image_path: impl AsRef<Path>,
        ) -> (i64, PathBuf) {
            let storage_path = storage_path.as_ref();
            let image_path = image_path.as_ref();
            let image_extension = image_path.extension().unwrap();
            let hash = get_file_hash_for_testing_by_path(image_path).await;
            let hash_str = u128_to_str((hash as u64) as u128);
            let image_path = storage_file_path(storage_path, hash_str, image_extension);
            // let storage_path = storage_path.join(&hash_str).with_extension(file_extension);
            (hash, image_path)
        }
    }
}

#[tokio::test]
async fn test_api_post_update_image_add() {
    use crate::auth::create_auth_cookie_str;
    use crate::{get_file_hash_for_testing_by_path, get_file_size};
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new(0, "test_api_post_update_file_add").await;

    let (user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let tmp_path = server.state.get_tmp_path().await;
    let storage_path = server.state.get_storage_path().await;

    // let get_storage_path = async |file_path_str: &str| {
    //     let file_path = Path::new(file_path_str);
    //     let file_extension = file_path.extension().unwrap();
    //     let hash = get_file_hash_for_testing_by_path(file_path_str).await;
    //     let hash_str = u128_to_str(hash as u128);
    //     let storage_path = storage_path.join(&hash_str).with_extension(file_extension);
    //     let file_path = storage_path.to_str().unwrap().to_string();
    //     (hash, PathBuf::from(file_path))
    // };

    // let txt_file = "/tmp/test.txt";
    let txt_file = "../flake.nix";
    let favicon_path = "../assets/favicon.ico";
    let favicon_size = get_file_size(favicon_path).await;
    // let tmp_path = Path::new(&tmp_path);
    // let path_org = path_org.join("test.txt");
    // fs::write(&path_org, "hello").await.unwrap();
    // let path_org = path_org.to_string_lossy().to_string();
    // let test_txt_path_org = server.state.get_tmp_path().await;

    // /tmp/test.txt
    let post1 = server
        .post_add("title", "description1", "tags1", session_key1)
        .await
        .unwrap();

    // let add_file = async |post_id: i64, files: &[&str]| {
    //     server
    //         .client
    //         .post_update_file_add(post_id, files.into_iter().map(|v| v.to_string()).collect())
    //         .header_add(
    //             header::COOKIE,
    //             create_auth_cookie_str(uuid_to_str(session_key1)),
    //         )
    //         .send()
    //         .await
    //         .into_json()
    //         .await
    // };
    let result = server
        .post_update_image_add(post1.id, &[txt_file], session_key1)
        .await;
    assert!(matches!(
        result,
        Err(PostUpdateImageAddErr::UnsupportedExtension(_))
    ));

    let result = server
        .post_update_image_add(post1.id, &[favicon_path, txt_file], session_key1)
        .await;

    let (favicon_hash, favicon_storage_path) = server
        .get_image_from_storage_path(storage_path, favicon_path)
        .await;

    assert!(!favicon_storage_path.exists());
    assert!(matches!(
        result,
        Err(PostUpdateImageAddErr::UnsupportedExtension(_))
    ));

    let favicon_size = get_file_size(favicon_path).await;
    let sizes = [
        (favicon_size - 1, favicon_size - 1),
        (favicon_size - 1, favicon_size),
        (favicon_size, favicon_size - 1),
    ];
    for (max, max_per_file) in sizes {
        let result = server
            .state
            .db
            .user_update_storage(0, user1.username.clone(), max, max_per_file)
            .await
            .unwrap();

        let result = server
            .post_update_image_add(post1.id, &[favicon_path, txt_file], session_key1)
            .await;
        assert!(matches!(
            result,
            Err(PostUpdateImageAddErr::ImageTooBig { .. })
        ));
    }

    let user = server.state.db.user_get_by_username("prime").await.unwrap();
    assert_eq!(user.used_storage_bytes, 0);

    let result = server
        .state
        .db
        .user_update_storage(0, user1.username.clone(), favicon_size, favicon_size)
        .await
        .unwrap();

    let files = server
        .post_update_image_add(post1.id, &[favicon_path], session_key1)
        .await
        .unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].extension, "ico");
    assert_eq!(files[0].size_bytes, favicon_size);
    assert_eq!(files[0].hash, favicon_hash);
    assert_eq!(files[0].proccesed, false);

    // assert_eq!(post1.file[0].width, favicon_size);

    let user = server.state.db.user_get_by_username("prime").await.unwrap();
    assert_eq!(user.used_storage_bytes, favicon_size);
}
