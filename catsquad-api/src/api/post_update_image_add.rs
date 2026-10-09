use crate::{
    api::user_update_pfp::remove_images,
    proccess_images::{storage_file_path, tmp_file_path},
    state::AppState,
};
use anyhow::anyhow;
use axum::{
    Extension, Json,
    extract::{Multipart, RawPathParams, State},
    http::StatusCode,
    response::IntoResponse,
};
use bytes::Bytes;
use catsquad_db::{DbPostUpdateImageAddErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    FileImage, FileImageAddErr, POST_UPDATE_IMAGE_ADD_PARAMS_FIELD_POST_ID, PostState,
    PostUpdateImageAddErr, PostUpdateImageAddParams, PostUpdateImageAddRes,
    SUPPORTED_IMAGE_EXTENSIONS, i64_to_str,
};
use futures::{Stream, TryStreamExt};
use futures_util::StreamExt;
use std::{
    hash::DefaultHasher,
    io,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    io::{AsyncWriteExt, BufWriter},
};

fn from_db_post_update_image_add(value: DbPostUpdateImageAddErr) -> PostUpdateImageAddErr {
    match value {
        DbPostUpdateImageAddErr::OutOfStorage => {
            PostUpdateImageAddErr::Image(FileImageAddErr::ImageTooBig {
                image_name: "uwnkown".to_string(),
                max: 0,
                got: 0,
            })
        }
        DbPostUpdateImageAddErr::ImageTooBig => {
            PostUpdateImageAddErr::Image(FileImageAddErr::ImageTooBig {
                image_name: "uwnkown".to_string(),
                max: 0,
                got: 0,
            })
        }
        DbPostUpdateImageAddErr::PostNotFound => PostUpdateImageAddErr::PostNotFound,
        DbPostUpdateImageAddErr::ImageAlreadyExists => PostUpdateImageAddErr::Duplicate,
        DbPostUpdateImageAddErr::Unauthorized => {
            PostUpdateImageAddErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateImageAddErr::Db(_) => PostUpdateImageAddErr::InternalServer,
        DbPostUpdateImageAddErr::InternalError(_) => PostUpdateImageAddErr::InternalServer,
    }
}

fn status_code(result: &Result<PostUpdateImageAddRes, PostUpdateImageAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateImageAddErr::NotImagesFound) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::Duplicate) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::ParamNotFoundPostId) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageAddErr::Image(FileImageAddErr::ReadingResolutionErr(..))) => {
            StatusCode::BAD_REQUEST
        }
        Err(PostUpdateImageAddErr::Image(FileImageAddErr::InvalidResolution { .. })) => {
            StatusCode::BAD_REQUEST
        }
        Err(PostUpdateImageAddErr::Image(FileImageAddErr::IoErr(_))) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Err(PostUpdateImageAddErr::Image(FileImageAddErr::StreamErr(_))) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Err(PostUpdateImageAddErr::Image(FileImageAddErr::ImageTooBig { .. })) => {
            StatusCode::BAD_REQUEST
        }
        Err(PostUpdateImageAddErr::Image(FileImageAddErr::ImageHasNoExtension(_))) => {
            StatusCode::BAD_REQUEST
        }
        Err(PostUpdateImageAddErr::Image(FileImageAddErr::UnsupportedExtension(_))) => {
            StatusCode::BAD_REQUEST
        }
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

#[derive(Clone, Debug)]
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
    max_imgs: usize,
    max_storage: u32,
    max_storage_per_image: u32,
    mut used_storage: u32,
) -> Result<Vec<Image>, FileImageAddErr> {
    let mut images = Vec::new();
    let storage_path = storage_path.as_ref();
    let tmp_path = tmp_path.as_ref();
    let mut index = 0;

    let mut inner = async || -> Result<(), FileImageAddErr> {
        while let Ok(Some(field)) = multipart.next_field().await {
            if index >= max_imgs {
                break;
            }
            let image_name = if let Some(image_name) = field.file_name() {
                image_name.to_owned()
            } else {
                continue;
            };

            let Some(extension) = Path::new(&image_name).extension().and_then(|v| v.to_str())
            else {
                return Err(FileImageAddErr::ImageHasNoExtension(image_name.to_string()));
            };
            let is_supported = SUPPORTED_IMAGE_EXTENSIONS
                .into_iter()
                .any(|v| *v == extension);
            if !is_supported {
                return Err(FileImageAddErr::UnsupportedExtension(extension.to_string()));
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
                        } => FileImageAddErr::ImageTooBig {
                            image_name: image_name.to_string(),
                            max: max_bytes,
                            got: got_bytes,
                        },
                        SaveImageErr::IoErr(err) => FileImageAddErr::IoErr(err.to_string()),
                        SaveImageErr::StreamErr(err) => FileImageAddErr::StreamErr(err.to_string()),
                    })?;

            let result = get_img_resolution(image.saved_path.to_str().unwrap()).await;
            let (width, height) = match result {
                Ok(v) => v,
                Err(err) => {
                    tokio::fs::remove_file(&image.saved_path)
                        .await
                        .map_err(|err| FileImageAddErr::IoErr(err.to_string()))?;
                    return Err(FileImageAddErr::ReadingResolutionErr(err.to_string()));
                }
            };

            if width == 0 || height == 0 {
                tokio::fs::remove_file(&image.saved_path)
                    .await
                    .map_err(|err| FileImageAddErr::IoErr(err.to_string()))?;
                return Err(FileImageAddErr::InvalidResolution { width, height });
            }

            used_storage += image.size_bytes;

            images.push(Image {
                saved_image: image,
                extension: extension.to_string(),
                width,
                height,
            });

            index += 1;
        }
        Ok(())
    };

    let result = inner().await;

    if let Err(err) = result {
        for image in images {
            let path = image.saved_image.saved_path;
            fs::remove_file(&path)
                .await
                .map_err(|err| FileImageAddErr::IoErr(err.to_string()))?;
        }
        return Err(err);
    };

    // result?;

    Ok(images)
}

#[derive(Clone, Debug)]
pub struct SavedImage {
    pub hash: i64,
    pub saved_path: PathBuf,
    pub size_bytes: u32,
    pub already_existed: bool,
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
    let hash = hasher.finish() as i64;
    let hash_str = i64_to_str(hash);
    trace!("hashing in prod {image_path_tmp:?} = {hash}");
    let mut already_existed = false;

    // uuid_to_str(uuid)

    let image_path = {
        let image_path = storage_file_path(storage_path, hash_str, extension);
        // let file_path = storage_path.join(&hash_str).with_extension(extension);
        if image_path.exists() {
            trace!("image removed");
            already_existed = true;
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
        already_existed,
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

    let inner = async || -> Result<PostUpdateImageAddRes, PostUpdateImageAddErr> {
        let req = params_req(params)?;

        let post_id = req.post_id;

        let Some(image) = parse_multipart(
            multipart,
            storage_path.as_path(),
            tmp_path,
            1,
            max_storage,
            max_storage_per_image,
            used_storage,
        )
        .await?
        .first()
        .cloned() else {
            return Err(PostUpdateImageAddErr::NotImagesFound);
        };

        let result = app
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
            .map_err(from_db_post_update_image_add);

        // TODO should this be inside db transaction?
        // this would have to error at the exact same time as someone else adding same image and succeeding
        if result.is_err() && !image.saved_image.already_existed {
            // thumbnail shouldnt exist if its new
            // so only remove the original file
            let _ = fs::remove_file(image.saved_image.saved_path.as_path())
                .await
                .inspect_err(|err| error!("error removing image {err} {image:#?}"));
        }

        result?;

        let image = FileImage {
            extension: image.extension,
            hash: image.saved_image.hash,
            proccesed: false, // TODO this might not be true, if image already existed
            size_bytes: image.saved_image.size_bytes,
            width: image.width,
            height: image.height,
        };

        // let mut post_images = Vec::new();
        // let mut latest_err = None;
        // let mut post = None;
        // for image in images {
        //     let result = app
        //         .db
        //         .post_update_image_add(
        //             time,
        //             user_username.clone(),
        //             post_id,
        //             image.saved_image.size_bytes,
        //             image.saved_image.hash.clone(),
        //             image.extension.clone(),
        //             image.width,
        //             image.height,
        //         )
        //         .await;

        //     // match result {
        //     //     Err(DbPost)

        //     //     _=>(),
        //     // }
        //     if let Err(err) = result {
        //         latest_err = Some(from_db_post_update_image_add(err));
        //         let _ = fs::remove_file(image.saved_image.saved_path.as_path())
        //             .await
        //             .inspect_err(|err| error!("error removing image {err} {image:#?}"));
        //     }

        //     post_images.push(FileImage {
        //         extension: image.extension,
        //         hash: image.saved_image.hash,
        //         proccesed: false,
        //         size_bytes: image.saved_image.size_bytes,
        //         width: image.width,
        //         height: image.height,
        //     });
        // }

        // if let Some(latest_err) = latest_err
        //     && post_images.is_empty()
        // {
        //     return Err(latest_err);
        // }

        // if post_images.is_empty() {
        //     return Err(PostUpdateImageAddErr::NotImagesFound);
        // }

        Ok(image)
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
    use catsquad_shared::{self as cs, PostUpdateImageAddRes, Uuid, i64_to_str, uuid_to_str};

    impl TestServer {
        pub async fn post_update_image_add(
            &self,
            post_id: i64,
            images: &[&str],
            session_token: Uuid,
        ) -> Result<PostUpdateImageAddRes, cs::PostUpdateImageAddErr> {
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
            let hash_str = i64_to_str(hash);
            let image_path = storage_file_path(storage_path, hash_str, image_extension);
            // let storage_path = storage_path.join(&hash_str).with_extension(file_extension);
            (hash, image_path)
        }
    }
}

#[tokio::test]
async fn test_api_post_update_image_add() {
    init_log();

    let server = crate::TestServer::new(0, "test_api_post_update_file_add").await;
    let input1_img = server.create_img_input(0).await;
    let input2_img = server.create_img_input(1).await;
    let input3_txt = "../flake.nix";

    let (user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1)
        .await
        .unwrap();
    server
        .post_update_state(post1.id, PostState::Active, session_key1)
        .await
        .unwrap();

    let post2 = server
        .post_add("title2", "description2", "tags2", session_key1)
        .await
        .unwrap();
    server
        .post_update_state(post2.id, PostState::Active, session_key1)
        .await
        .unwrap();

    // assert invalid input
    {
        let result = server
            .post_update_image_add(post1.id, &[input3_txt], session_key1)
            .await;
        assert!(matches!(
            result,
            Err(PostUpdateImageAddErr::Image(
                FileImageAddErr::UnsupportedExtension(_)
            ))
        ));
    }

    // assert half invalid input
    {
        let result = server
            .post_update_image_add(
                post1.id,
                &[input3_txt, input1_img.saved_path.to_str().unwrap()],
                session_key1,
            )
            .await;
        assert!(matches!(
            result,
            Err(PostUpdateImageAddErr::Image(
                FileImageAddErr::UnsupportedExtension(_)
            ))
        ));

        assert!(!input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());
        assert!(!input2_img.storage_path.exists());
        assert!(!input2_img.thumbnail_path.exists());
    }

    // try different combinations of user storage limits when uploading image
    {
        let sizes = [
            (input1_img.size - 1, input1_img.size - 1),
            (input1_img.size - 1, input1_img.size),
            (input1_img.size, input1_img.size - 1),
        ];
        for (max, max_per_file) in sizes {
            server
                .state
                .db
                .user_update_storage(0, user1.username.clone(), max, max_per_file)
                .await
                .unwrap();

            let result = server
                .post_update_image_add(
                    post1.id,
                    &[input1_img.saved_path.to_str().unwrap(), input3_txt],
                    session_key1,
                )
                .await;
            assert!(matches!(
                result,
                Err(PostUpdateImageAddErr::Image(
                    FileImageAddErr::ImageTooBig { .. }
                ))
            ));
        }
    }

    // upload image successfully
    {
        server
            .state
            .db
            .user_update_storage(
                0,
                user1.username.clone(),
                input1_img.size * 3,
                input1_img.size * 3,
            )
            .await
            .unwrap();

        let image = server
            .post_update_image_add(
                post1.id,
                &[input1_img.saved_path.to_str().unwrap()],
                session_key1,
            )
            .await
            .unwrap();

        assert_eq!(image.extension, "png");
        assert_eq!(image.size_bytes, input1_img.size);
        assert_eq!(image.hash, input1_img.hash);
        assert_eq!(image.proccesed, false);
        assert!(input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());
        assert!(!input2_img.storage_path.exists());
        assert!(!input2_img.thumbnail_path.exists());
    }

    // assert duplicate
    {
        let result1 = server
            .post_update_image_add(
                post1.id,
                &[input1_img.saved_path.to_str().unwrap()],
                session_key1,
            )
            .await;
        let result2 = server
            .post_update_image_add(
                post1.id,
                &[
                    input1_img.saved_path.to_str().unwrap(),
                    input2_img.saved_path.to_str().unwrap(),
                ],
                session_key1,
            )
            .await;

        let post1 = server.post_get_by_id(post1.id, session_key1).await.unwrap();

        assert!(matches!(result1, Err(PostUpdateImageAddErr::Duplicate)));
        assert!(matches!(result2, Err(PostUpdateImageAddErr::Duplicate)));
        assert!(input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());
        assert!(!input2_img.storage_path.exists()); // second img still gets saved
        assert!(!input2_img.thumbnail_path.exists());
        assert_eq!(post1.images.len(), 1);
        assert_eq!(post1.images[0].hash, input1_img.hash);
    }
}
