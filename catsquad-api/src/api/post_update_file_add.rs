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
use catsquad_db::{DbPostFile, DbPostUpdateFileAddErr, DbUser, id_to_string};
use catsquad_log::prelude::*;
use catsquad_shared::{
    POST_UPDATE_FILE_ADD_PARAMS_FIELD_POST_KEY, PostFile, PostUpdateFileAddErr,
    PostUpdateFileAddParams, SUPPORTED_FILE_EXTENSIONS,
};
use futures::{Stream, TryStreamExt};
use futures_util::StreamExt;
use tokio::{
    // fs::File,
    fs,
    io::{AsyncWriteExt, BufWriter},
};

use crate::{
    api::{email_change_add::from_db_email_change, post_add::from_db_post},
    state::AppState,
};

fn from_db_post_update_file_add(value: DbPostUpdateFileAddErr) -> PostUpdateFileAddErr {
    match value {
        DbPostUpdateFileAddErr::OutOfStorage => PostUpdateFileAddErr::FileTooBig {
            file_name: "uwnkown".to_string(),
            max: 0,
            got: 0,
        },
        DbPostUpdateFileAddErr::FileTooBig => PostUpdateFileAddErr::FileTooBig {
            file_name: "uwnkown".to_string(),
            max: 0,
            got: 0,
        },
        DbPostUpdateFileAddErr::PostNotFound => PostUpdateFileAddErr::PostNotFound,
        DbPostUpdateFileAddErr::FileAlreadyExists => PostUpdateFileAddErr::Duplicate,
        DbPostUpdateFileAddErr::Unauthorized => {
            PostUpdateFileAddErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateFileAddErr::Db(_) => PostUpdateFileAddErr::InternalServer,
    }
}

fn status_code(result: &Result<Vec<PostFile>, PostUpdateFileAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateFileAddErr::NotFilesFound) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileAddErr::Duplicate) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileAddErr::ParamNotFoundPostId) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileAddErr::ReadingResolutionErr(..)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileAddErr::InvalidResolution { .. }) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileAddErr::IoErr(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(PostUpdateFileAddErr::StreamErr(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(PostUpdateFileAddErr::FileTooBig { .. }) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileAddErr::FileHasNoExtension(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileAddErr::UnsupportedExtension(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileAddErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateFileAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateFileAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn params_req(value: RawPathParams) -> Result<PostUpdateFileAddParams, PostUpdateFileAddErr> {
    value
        .iter()
        .find(|(name, _)| *name == POST_UPDATE_FILE_ADD_PARAMS_FIELD_POST_KEY)
        .ok_or(PostUpdateFileAddErr::ParamNotFoundPostId)
        .map(|(_, value)| PostUpdateFileAddParams {
            post_key: value.to_string(),
        })
}

pub struct File {
    pub saved_file: SavedFile,
    pub extension: String,
    pub width: u32,
    pub height: u32,
    // pub saved_path: PathBuf,
    // pub size_bytes: u64,
}

pub async fn parse_multipart(
    mut multipart: Multipart,
    storage_path: impl AsRef<str>,
    tmp_path: impl AsRef<str>,
    max_storage: u64,
    max_storage_per_file: u64,
    mut used_storage: u64,
) -> Result<Vec<File>, PostUpdateFileAddErr> {
    let mut files = Vec::new();
    let storage_path = storage_path.as_ref();
    let tmp_path = tmp_path.as_ref();

    let mut inner = async || -> Result<(), PostUpdateFileAddErr> {
        while let Ok(Some(field)) = multipart.next_field().await {
            let file_name = if let Some(file_name) = field.file_name() {
                file_name.to_owned()
            } else {
                continue;
            };

            let Some(extension) = Path::new(&file_name).extension().and_then(|v| v.to_str()) else {
                return Err(PostUpdateFileAddErr::FileHasNoExtension(
                    file_name.to_string(),
                ));
            };
            let is_supported = SUPPORTED_FILE_EXTENSIONS
                .into_iter()
                .any(|v| *v == extension);
            if !is_supported {
                return Err(PostUpdateFileAddErr::UnsupportedExtension(
                    extension.to_string(),
                ));
            }

            let storage_left = max_storage.saturating_sub(used_storage);
            let storage_per_file = if storage_left < max_storage_per_file {
                storage_left
            } else {
                max_storage_per_file
            };

            let stream = field.map_err(io::Error::other);
            let file =
                handle_file_saving(stream, extension, storage_path, storage_per_file, tmp_path)
                    .await
                    .map_err(|err| match err {
                        SaveFileErr::FileTooBig {
                            got_bytes,
                            max_bytes,
                        } => PostUpdateFileAddErr::FileTooBig {
                            file_name: file_name.to_string(),
                            max: max_bytes,
                            got: got_bytes,
                        },
                        SaveFileErr::IoErr(err) => PostUpdateFileAddErr::IoErr(err.to_string()),
                        SaveFileErr::StreamErr(err) => {
                            PostUpdateFileAddErr::StreamErr(err.to_string())
                        }
                    })?;

            let result = get_img_resolution(file.saved_path.to_str().unwrap()).await;
            let (width, height) = match result {
                Ok(v) => v,
                Err(err) => {
                    tokio::fs::remove_file(&file.saved_path)
                        .await
                        .map_err(|err| PostUpdateFileAddErr::IoErr(err.to_string()))?;
                    return Err(PostUpdateFileAddErr::ReadingResolutionErr(err.to_string()));
                }
            };

            if width == 0 || height == 0 {
                tokio::fs::remove_file(&file.saved_path)
                    .await
                    .map_err(|err| PostUpdateFileAddErr::IoErr(err.to_string()))?;
                return Err(PostUpdateFileAddErr::InvalidResolution { width, height });
            }

            used_storage += file.size_bytes;

            files.push(File {
                saved_file: file,
                extension: extension.to_string(),
                width,
                height,
            });
        }
        Ok(())
    };

    let result = inner().await;

    if let Err(err) = result {
        for file in files {
            let path = file.saved_file.saved_path;
            fs::remove_file(&path)
                .await
                .map_err(|err| PostUpdateFileAddErr::IoErr(err.to_string()))?;
        }
        return Err(err);
    };

    // result?;

    Ok(files)
}

pub struct SavedFile {
    pub hash: String,
    pub saved_path: PathBuf,
    pub size_bytes: u64,
}

#[derive(thiserror::Error, Debug)]
pub enum SaveFileErr {
    #[error("max file size {max_bytes} bytes, upload stopped at {got_bytes} bytes")]
    FileTooBig { max_bytes: u64, got_bytes: u64 },

    #[error("io err {0}")]
    IoErr(#[from] std::io::Error),

    #[error(transparent)]
    StreamErr(#[from] anyhow::Error),
}

pub async fn handle_file_saving<S, StreamErr>(
    mut stream: S,
    extension: impl AsRef<str>,
    save_path: impl AsRef<str>,
    max_storage_per_file: u64,
    tmp_path: impl AsRef<str>,
    // used_storage: usize,
    // max_storage: usize,
) -> Result<SavedFile, SaveFileErr>
where
    S: StreamExt + Stream<Item = Result<Bytes, StreamErr>> + Unpin,
    StreamErr: Sync + Send,
    SaveFileErr: From<StreamErr>, // S::Item: Error + Try,
{
    use rand::distr::SampleString;
    use std::hash::Hasher;
    let tmp_path = tmp_path.as_ref();
    let extension = extension.as_ref();
    let mut tmp_name = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 16);
    tmp_name.push_str("_upload");
    let file_path_tmp = Path::new(tmp_path)
        .join(&tmp_name)
        .with_extension(extension);
    // let file_path_tmp = Path::new("/tmp/").join(&tmp_name).with_extension("part");
    // extension.as_ref()
    let file = fs::File::create(&file_path_tmp).await?;
    let mut file = BufWriter::new(file);

    let mut hasher = DefaultHasher::default();
    // let mut hasher = GxHasher::with_seed(0);
    let mut size = 0_u64;

    while let Some(value) = stream.next().await {
        let bytes = value?;
        size += bytes.len() as u64;
        if size > max_storage_per_file {
            file.flush().await?;
            drop(file);
            tokio::fs::remove_file(file_path_tmp).await?;
            return Err(SaveFileErr::FileTooBig {
                max_bytes: max_storage_per_file,
                got_bytes: size,
            });
        }
        hasher.write(&bytes);
        file.write(&bytes).await?;
    }

    file.flush().await?;
    let hash = hasher.finish().to_string();
    trace!("hashing in prod {file_path_tmp:?} = {hash}");

    let file_path = {
        let file_path = Path::new(save_path.as_ref())
            .join(&hash)
            .with_extension(extension);
        if file_path.exists() {
            trace!("file removed");
            tokio::fs::remove_file(file_path_tmp).await?;
        } else {
            trace!("file moved");
            // TODO remove file on any error
            tokio::fs::rename(&file_path_tmp, &file_path)
                .await
                .inspect_err(|err| {
                    error!(
                        "move err from {file_path_tmp:?} to {}/{} {err}",
                        std::env::current_dir().unwrap().to_str().unwrap(),
                        file_path.clone().to_str().unwrap(),
                    )
                })?;
        }
        file_path
    };

    Ok(SavedFile {
        hash,
        size_bytes: size,
        saved_path: file_path,
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

pub async fn post_update_file_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    // Form(req): Form<EmailChangeUpdateNewAddReq>,
    params: axum::extract::RawPathParams,
    multipart: Multipart,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let max_storage = db_user.max_storage_bytes;
    let max_storage_per_file = db_user.max_storage_per_file_bytes;
    let user_id = db_user.id.clone();
    let used_storage = db_user.used_storage_bytes;
    let storage_path = app.get_storage_path().await;
    let tmp_path = app.get_tmp_path().await;

    let inner = async || -> Result<Vec<PostFile>, PostUpdateFileAddErr> {
        let req = params_req(params)?;

        let post_key = req.post_key;

        let files = parse_multipart(
            multipart,
            storage_path,
            tmp_path,
            max_storage,
            max_storage_per_file,
            used_storage,
        )
        .await?;

        let mut post_files = Vec::new();
        // let mut post = None;
        for file in files {
            let _result = app
                .db
                .post_update_file_add(
                    time,
                    user_id.clone(),
                    post_key.clone(),
                    file.saved_file.size_bytes,
                    file.saved_file.hash.clone(),
                    file.extension.clone(),
                    file.width,
                    file.height,
                )
                .await
                .map_err(from_db_post_update_file_add)?;

            // post = Some(result);
            post_files.push(PostFile {
                extension: file.extension,
                hash: file.saved_file.hash,
                proccesed: false,
                size_bytes: file.saved_file.size_bytes,
                width: file.width,
                height: file.height,
            });
        }

        // let post = post.ok_or_else(|| PostUpdateFileAddErr::NotFilesFound)?;

        if post_files.is_empty() {
            return Err(PostUpdateFileAddErr::NotFilesFound);
        }

        Ok(post_files)
        // Ok(from_db_post(post))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[tokio::test]
async fn test_post_update_file_add() {
    use crate::auth::create_auth_cookie_str;
    use crate::{get_file_hash_for_testing_by_path, get_file_size};
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new().await;

    let (user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let tmp_path = server.state.get_tmp_path().await;
    let storage_path = server.state.get_storage_path().await;

    let get_storage_path = async |file_path_str: &str| {
        let file_path = Path::new(file_path_str);
        let file_extension = file_path.extension().unwrap();
        let hash = get_file_hash_for_testing_by_path(file_path_str).await;
        let storage_path = Path::new(&storage_path)
            .join(&hash)
            .with_extension(file_extension);
        let file_path = storage_path.to_str().unwrap().to_string();
        (hash, PathBuf::from(file_path))
    };

    let txt_file = "/tmp/test.txt";
    let favicon_path = "../assets/favicon.ico";
    let favicon_size = get_file_size(favicon_path).await;
    // let tmp_path = Path::new(&tmp_path);
    // let path_org = path_org.join("test.txt");
    // fs::write(&path_org, "hello").await.unwrap();
    // let path_org = path_org.to_string_lossy().to_string();
    // let test_txt_path_org = server.state.get_tmp_path().await;

    // /tmp/test.txt
    let post1 = server
        .client
        .post_add("title", "description1", "tags1")
        .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    let add_file = async |post_key: &str, files: &[&str]| {
        server
            .client
            .post_update_file_add(post_key, files.into_iter().map(|v| v.to_string()).collect())
            .header_add(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
            .send()
            .await
            .into_res()
            .await
    };
    let result = add_file(&post1.key, &[txt_file]).await;
    assert!(matches!(
        result,
        Err(PostUpdateFileAddErr::UnsupportedExtension(_))
    ));

    let result = add_file(&post1.key, &[favicon_path, txt_file]).await;

    let (favicon_hash, favicon_storage_path) = get_storage_path(favicon_path).await;

    assert!(!favicon_storage_path.exists());
    assert!(matches!(
        result,
        Err(PostUpdateFileAddErr::UnsupportedExtension(_))
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
            .user_update_storage(0, user1.id.clone(), max, max_per_file)
            .await
            .unwrap();

        let result = add_file(&post1.key, &[favicon_path, txt_file]).await;
        assert!(matches!(
            result,
            Err(PostUpdateFileAddErr::FileTooBig { .. })
        ));
    }

    let user = server.state.db.user_get_by_username("prime").await.unwrap();
    assert_eq!(user.used_storage_bytes, 0);

    let result = server
        .state
        .db
        .user_update_storage(0, user1.id.clone(), favicon_size, favicon_size)
        .await
        .unwrap();

    let files = add_file(&post1.key, &[favicon_path]).await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].extension, "ico");
    assert_eq!(files[0].size_bytes, favicon_size);
    assert_eq!(files[0].hash, favicon_hash);
    assert_eq!(files[0].proccesed, false);

    // assert_eq!(post1.file[0].width, favicon_size);

    let user = server.state.db.user_get_by_username("prime").await.unwrap();
    assert_eq!(user.used_storage_bytes, favicon_size);
}
