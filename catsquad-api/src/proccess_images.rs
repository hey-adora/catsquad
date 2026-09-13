use anyhow::anyhow;
use catsquad_db::Db;
use catsquad_log::prelude::*;
use catsquad_shared::{Uuid, i64_to_str, uuid_to_str};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub fn scale_res_by_width(width: u32, height: u32, new_width: u32) -> (u32, u32) {
    let ratio = height as f32 / width as f32;
    let new_height = ratio * new_width as f32;
    (new_width as u32, new_height as u32)
}

#[test]
fn test_scale_res_by_width() {
    let result = scale_res_by_width(1920, 1080, 1280);
    assert_eq!(result, (1280, 720));

    let result = scale_res_by_width(1080, 1920, 1280);
    assert_eq!(result, (1280, 2275));
}

pub fn scale_res_by_height(width: u32, height: u32, new_height: u32) -> (u32, u32) {
    let ratio = width as f32 / height as f32;
    let new_width = ratio * new_height as f32;
    (new_width as u32, new_height as u32)
}

#[test]
fn test_scale_res_by_height() {
    let result = scale_res_by_height(1920, 1080, 720);
    assert_eq!(result, (1280, 720));
    let result = scale_res_by_height(1080, 1920, 720);
    assert_eq!(result, (405, 720));
}

pub fn scale_resolution(width: u32, height: u32, clamp: u32) -> (u32, u32) {
    match (width, height) {
        (width, height) if width > clamp && width >= height => {
            scale_res_by_width(width, height, clamp)
        }
        (width, height) if height > clamp && height >= width => {
            scale_res_by_height(width, height, clamp)
        }
        (width, height) => (width, height),
    }
}

#[test]
fn test_scale_resolution() {
    let result = scale_resolution(1920, 1080, 1280);
    assert_eq!(result, (1280, 720));
    let result = scale_resolution(1080, 1920, 1280);
    assert_eq!(result, (720, 1280));
    let result = scale_resolution(1280, 720, 1280);
    assert_eq!(result, (1280, 720));
}

pub fn storage_file_path(
    storage_path: impl AsRef<Path>,
    name: impl AsRef<str>,
    extension: impl AsRef<OsStr>,
) -> PathBuf {
    storage_path
        .as_ref()
        .join(name.as_ref())
        .with_extension(extension.as_ref())
}

pub fn tmp_file_path(
    tmp_path: impl AsRef<Path>,
    name: impl AsRef<str>,
    extension: impl AsRef<OsStr>,
) -> PathBuf {
    tmp_path
        .as_ref()
        .join(name.as_ref())
        .with_extension(extension.as_ref())
}

pub fn thumbnail_file_path(storage_path: impl AsRef<Path>, name: impl AsRef<str>) -> PathBuf {
    let name = name.as_ref();
    storage_path
        .as_ref()
        .join(format!("{}_thumbnail.webp", name))
}

// pub fn disect_file_path<'a, T>(file_path: T) -> Option<(&'a OsStr, &'a OsStr)>
// where
//     T: AsRef<Path> + 'a,
// {
//     let file_path = file_path.as_ref();
//     Some((file_path.file_name()?, file_path.extension()?))
//     // tmp_path.as_ref().join(format!("{}_thumbnail.webp", name))
// }

#[test]
fn test_paths() {
    assert_eq!(
        storage_file_path("/tmp", "one", "webp").to_str().unwrap(),
        "/tmp/one.webp"
    );
    assert_eq!(
        tmp_file_path("/tmp", "one", "webp").to_str().unwrap(),
        "/tmp/one.webp"
    );
    assert_eq!(
        thumbnail_file_path("/tmp", "one").to_str().unwrap(),
        "/tmp/one_thumbnail.webp"
    );

    // let result = scale_resolution(1080, 1920, 1280);
    // assert_eq!(result, (720, 1280));
    // let result = scale_resolution(1280, 720, 1280);
    // assert_eq!(result, (1280, 720));
}

// fn storage_file_path(
//     storage_path: impl AsRef<Path>,
//     name: impl AsRef<str>,
//     extension: impl AsRef<str>,
// ) -> PathBuf {
//     storage_path
//         .as_ref()
//         .join(name.as_ref())
//         .with_extension(extension.as_ref())
// }

// #[test]
// fn test_to_thumbnail_path() {
//     let file = DBUserPostFile {
//         proccesed: false,
//         extension: String::from("webp"),
//         hash: String::from("one"),
//         size_bytes: 1,
//         width: 10,
//         height: 10,
//     };
//     let file_path = file.to_file_path("/tmp");
//     let thumbnail_path = to_thumbnail_path(file_path).unwrap();
//     assert_eq!(
//         "/tmp/one_thumbnail_default.webp",
//         thumbnail_path.to_str().unwrap()
//     );
// }

#[derive(Debug, Clone)]
pub struct ProccesedFileResult {
    pub path: PathBuf,
    pub already_existed: bool,
}

pub async fn proccess_post_file(
    storage_path: impl AsRef<Path>,
    file_hash: i64,
    file_extension: impl AsRef<OsStr>,
    width: u32,
    height: u32,
    resolution_limit: u32,
) -> Result<ProccesedFileResult, anyhow::Error> {
    // let storage_file_path = storage_file_path.as_ref();
    // let file_name = storage_file_path
    //     .file_name()
    //     .ok_or_else(|| anyhow!("failed to get file_name"))?;
    // let file_extension = storage_file_path
    //     .file_name()
    //     .ok_or_else(|| anyhow!("failed to get file_extension"))?;
    // let storage_file_path = storage_file_path
    //     .to_str()
    //     .ok_or_else(|| anyhow!("invalid filename"))?;

    // TODO fix performance, strings and format and Path are bad

    let file_hash_str = i64_to_str(file_hash);
    let storage_path = storage_path.as_ref();

    // thumbnail path
    let thumbnail_path = {
        let thumbnail_path = thumbnail_file_path(storage_path, &file_hash_str);

        if thumbnail_path.exists() {
            return Ok(ProccesedFileResult {
                path: thumbnail_path,
                already_existed: true,
            });
        }

        thumbnail_path
    };

    // create thumbnail
    let arg_output_path = {
        let (new_width, new_height) = scale_resolution(width, height, resolution_limit);

        let arg_input_path = storage_file_path(storage_path, file_hash_str, file_extension);
        let arg_input_path_str = arg_input_path
            .to_str()
            .ok_or_else(|| anyhow!("failed to get input_path"))?;

        let arg_output_path = thumbnail_path;
        let arg_output_path_str = arg_output_path
            .to_str()
            .ok_or_else(|| anyhow!("failed to get output_path"))?;

        let arg_scale = format!("scale={new_width}:{new_height}");

        let mut command = tokio::process::Command::new("ffmpeg");
        let command = command.args(&[
            "-y",
            "-i",
            arg_input_path_str,
            "-vf",
            &arg_scale,
            arg_output_path_str,
        ]);
        trace!("running command {command:?}");
        let result = command.output().await?;

        let result = String::from_utf8(result.stdout)?;
        let result = result.trim();
        trace!("command output {result}");

        arg_output_path
    };

    if !arg_output_path.exists() {
        return Err(anyhow!("failed to create thumbnail {arg_output_path:?}"));
    }

    // // let output_path = to_thumbnail_path(arg_input_path)?;

    // if output_path.exists() {
    //     return Ok(ProccesedFileResult {
    //         path: output_path,
    //         already_existed: true,
    //     });
    // }

    // let arg_output_path = output_path
    //     .to_str()
    //     .ok_or_else(|| anyhow!("invalid filename"))?;

    // let arg_scale = format!("scale={new_width}:{new_height}");

    // let result = String::from_utf8(result.stdout)?;
    // let result = result.trim();
    // trace!("command output {result}");

    Ok(ProccesedFileResult {
        path: arg_output_path,
        already_existed: false,
    })
}

#[cfg(test)]
#[tokio::test]
async fn test_proccess_post_file_() {
    use tokio::fs;

    init_log();

    let img_path = "../assets/upload.svg";

    let server = crate::TestServer::new(0, "test_proccess_post_file").await;
    let storage_path = server.state.get_storage_path().await;

    let storage_file_path = storage_file_path(storage_path.clone(), "0", "svg");

    let result = proccess_post_file(storage_path.clone(), 0, "jpg", 10, 10, 10).await;
    assert!(matches!(result, Err(_)));

    fs::copy(img_path, storage_file_path).await.unwrap();

    let result = proccess_post_file(storage_path.clone(), 0, "svg", 10, 10, 10).await;
    trace!("{result:#?}");
    assert!(matches!(result, Ok(_)));

    // let img_path = "../assets/upload.svg";
    // let tmp_path = "/tmp/test_proccess_post_file.svg";
    // tokio::fs::copy(img_path, tmp_path).await.unwrap();
    // let (width, height) = get_img_resolution(img_path).await.unwrap();
    // let output = proccess_post_file(tmp_path, width, height, 1280)
    //     .await
    //     .unwrap();
    // assert!(output.path.exists());
    // assert_eq!(output.already_existed, false);
    // let output = proccess_post_file(tmp_path, width, height, 1280)
    //     .await
    //     .unwrap();
    // assert!(output.path.exists());
    // assert_eq!(output.already_existed, true);
    // tokio::fs::remove_file(output.path).await.unwrap();
}

pub async fn proccess_post_files(
    time: u64,
    db: Db,
    storage_path: impl AsRef<Path>,
    resolution_limit: u32,
) -> Result<(), anyhow::Error> {
    let storage_path = storage_path.as_ref();
    let images = db.file_image_get_unproccesed().await?;
    for image in images {
        let result = proccess_post_file(
            storage_path,
            image.hash,
            image.extension,
            image.width,
            image.height,
            resolution_limit,
        )
        .await?;
        db.file_image_update_proccsed(time, image.hash).await?;
    }
    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_proccess_post_files() {
    init_log();

    let server = crate::TestServer::new(0, "test_proccess_post_files").await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1)
        .await
        .unwrap();

    let files = server
        .post_update_file_add(post1.id, &["../assets/favicon.ico"], session_key1)
        .await
        .unwrap();

    let file = files[0].clone();
    let file_hash_str = i64_to_str(file.hash);
    let storage_path = server.state.get_storage_path().await;
    let file_path = storage_file_path(
        storage_path.clone(),
        file_hash_str.clone(),
        file.extension.clone(),
    );
    let file_thumbnail_path = thumbnail_file_path(storage_path.clone(), file_hash_str);

    trace!("{file_path:?}");
    trace!("{file_thumbnail_path:?}");

    assert!(file_path.exists());
    assert!(!file_thumbnail_path.exists());

    proccess_post_files(0, server.state.db.clone(), storage_path.clone(), 1280)
        .await
        .unwrap();

    assert!(file_path.exists());
    assert!(file_thumbnail_path.exists());

    proccess_post_files(0, server.state.db.clone(), storage_path, 1280)
        .await
        .unwrap();

    assert!(file_path.exists());
    assert!(file_thumbnail_path.exists());
    // proccess_post_files
}
// #[tokio::test]
// async fn test_proccess_post_files() {
//     // TODO delete files after test ends
//     crate::init_test_log();
//     const FILES_PATH: &str = "/tmp/test_proccess_post_files";
//     let app = crate::api::tests::ApiTestApp::new_with_exp_and_files(1, FILES_PATH).await;
//     let img_path = "../assets/upload.svg";

//     {
//         let auth_token = app
//             .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
//             .await
//             .unwrap();

//         let user = app.state.db.get_user_by_username("hey").await.unwrap();

//         let post = app
//             .add_post(0, &auth_token, "title1", "cat", "one")
//             .await
//             .unwrap();

//         let post = app
//             .add_post_file(0, &auth_token, post.key.clone(), img_path)
//             .await
//             .unwrap();
//     }

//     {
//         proccess_post_files(app.state.db.clone(), FILES_PATH, 1280)
//             .await
//             .unwrap();
//     }

//     {
//         let posts = app.state.db.get_post_unproccesed().await.unwrap();

//         for post in posts {
//             for file in post.file {
//                 let file_path = file.to_file_path(FILES_PATH);
//                 let thumbnail_path = file.to_thumbnail_path(FILES_PATH);

//                 assert_eq!(file.proccesed, true);
//                 assert!(file_path.exists());
//                 assert!(thumbnail_path.exists());
//                 tokio::fs::remove_file(file_path).await.unwrap();
//                 tokio::fs::remove_file(thumbnail_path).await.unwrap();
//             }
//         }
//     }
// }
