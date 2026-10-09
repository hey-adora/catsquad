use crate::{api::user_update_pfp::remove_images, state::AppState};
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateImageRemoveErr, DbUser};
use catsquad_shared::{PostUpdateImageRemoveErr, PostUpdateImageRemoveReq};

fn from_db_post_update_image_remove_err(
    value: DbPostUpdateImageRemoveErr,
) -> PostUpdateImageRemoveErr {
    match value {
        DbPostUpdateImageRemoveErr::PostNotFound => PostUpdateImageRemoveErr::PostNotFound,
        DbPostUpdateImageRemoveErr::Unauthorized => {
            PostUpdateImageRemoveErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateImageRemoveErr::FileNotFound => PostUpdateImageRemoveErr::FileNotFound,
        DbPostUpdateImageRemoveErr::Db(_) => PostUpdateImageRemoveErr::InternalServer,
        DbPostUpdateImageRemoveErr::InternalError(_) => PostUpdateImageRemoveErr::InternalServer,
        DbPostUpdateImageRemoveErr::ImageRemove(_) => PostUpdateImageRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostUpdateImageRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateImageRemoveErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateImageRemoveErr::FileNotFound) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateImageRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_image_remove(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateImageRemoveReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), PostUpdateImageRemoveErr> {
        let user_username = db_user.username.clone();
        let post_id = req.post_id;
        let hash = req.hash;
        let storage_path = app.get_storage_path().await;

        let callback_remove_file = async move |hash: i64, extension: &str| {
            remove_images(storage_path, hash, extension).await;
        };

        app.db
            .post_update_file_remove(time, callback_remove_file, user_username, post_id, hash)
            .await
            .map_err(from_db_post_update_image_remove_err)?;

        Ok(())
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn post_update_file_remove(
            &self,
            post_id: i64,
            file_hash: i64,
            session_token: Uuid,
        ) -> Result<(), cs::PostUpdateImageRemoveErr> {
            self.client
                .post_update_image_remove(post_id, file_hash)
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
async fn test_api_post_update_file_remove() {
    use catsquad_log::prelude::*;
    use catsquad_shared::PostState;
    init_log();

    let server = crate::TestServer::new(0, "test_api_post_update_file_remove").await;
    let input1_img = server.create_img_input(0).await;
    let input2_img = server.create_img_input(1).await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (_user2, session_key2) = server
        .user_add_full("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1)
        .await
        .unwrap();

    server
        .post_update_state(post1.id, PostState::Active, session_key1)
        .await
        .unwrap();

    // add imgs
    {
        for img in [input1_img.clone(), input2_img.clone()] {
            server
                .post_update_image_add(post1.id, &[img.saved_path.to_str().unwrap()], session_key1)
                .await
                .unwrap();
        }

        let post1 = server.post_get_by_id(post1.id, session_key1).await.unwrap();
        assert_eq!(post1.images.len(), 2);
        assert_eq!(post1.images[0].hash, input1_img.hash);
        assert_eq!(post1.images[1].hash, input2_img.hash);
        // let file1_hash = post1.images[0].hash;
    }

    // assert error file not found
    {
        let result = server
            .post_update_file_remove(post1.id, 0, session_key1)
            .await;
        assert!(matches!(
            result,
            Err(PostUpdateImageRemoveErr::FileNotFound)
        ));
    }

    // assert error post not found
    {
        let result = server
            .post_update_file_remove(0, input1_img.hash, session_key1)
            .await;
        assert!(matches!(
            result,
            Err(PostUpdateImageRemoveErr::PostNotFound)
        ));
    }

    // assert error unauthorized
    {
        let result = server
            .post_update_file_remove(post1.id, input1_img.hash, session_key2)
            .await;
        assert!(matches!(
            result,
            Err(PostUpdateImageRemoveErr::Unauthorized(_))
        ));
    }

    // assert success
    {
        let post1 = server.post_get_by_id(post1.id, session_key1).await.unwrap();
        assert_eq!(post1.images.len(), 2);
        assert_eq!(post1.images[0].hash, input1_img.hash);
        assert_eq!(post1.images[1].hash, input2_img.hash);
        assert!(input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());

        server
            .post_update_file_remove(post1.id, input1_img.hash, session_key1)
            .await
            .unwrap();

        let post1 = server.post_get_by_id(post1.id, session_key1).await.unwrap();
        assert_eq!(post1.images.len(), 1);
        assert_eq!(post1.images[0].hash, input2_img.hash);
        assert!(!input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());
    }
    // server
    //     .post_update_file_remove(post1.id, input1_img.hash, session_key1)
    //     .await
    //     .unwrap();

    // let result = server.post_get_by_id(post1.id, session_key1).await.unwrap();

    // assert_eq!(result.images.len(), 0);
}
