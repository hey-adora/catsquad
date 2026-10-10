use crate::{api::user_update_pfp::remove_images, state::AppState};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbPostRemoveErr, DbUser};
use catsquad_shared::{PostRemoveErr, PostRemoveParams};

fn from_db_post_remove_err(value: DbPostRemoveErr) -> PostRemoveErr {
    match value {
        DbPostRemoveErr::NotFound(_) => PostRemoveErr::PostNotFound,
        DbPostRemoveErr::Unauthorized => PostRemoveErr::Unauthorized("unauthorized".to_string()),
        DbPostRemoveErr::Db(_) => PostRemoveErr::InternalServer,
        DbPostRemoveErr::Image(_) => PostRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostRemoveErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_remove(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Path(req): Path<PostRemoveParams>,
) -> impl IntoResponse {
    let inner = async || -> Result<(), PostRemoveErr> {
        let user_username = db_user.username.clone();
        let post_id = req.post_id;
        let time = app.get_time_micro();
        let storage_path = app.get_storage_path().await;

        let callback_remove_image = async move |hash: i64, extension: &str| {
            remove_images(storage_path.as_path(), hash, extension).await;
        };

        app.db
            .post_remove(time, callback_remove_image, user_username, post_id)
            .await
            .map_err(from_db_post_remove_err)?;

        Ok(())
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
        pub async fn post_remove(
            &self,
            post_id: i64,
            session_token: Uuid,
        ) -> Result<(), cs::PostRemoveErr> {
            self.client
                .post_remove(post_id)
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
async fn test_api_post_remove() {
    use catsquad_log::prelude::*;
    use catsquad_shared::{PostGetByKeyErr, PostState};

    init_log();

    let server = crate::TestServer::new(0, "test_api_post_remove").await;
    let storage_path = server.state.get_storage_path().await;

    let input1_img = server.create_img_input(0).await;
    let input2_img = server.create_img_input(1).await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (_user2, session_key2) = server
        .user_add_full("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
        .await;

    let (post1, post2) = {
        let post1 = server
            .post_add("title", "description1", "tags1", session_key1)
            .await
            .unwrap();

        server
            .post_update_state(post1.id, PostState::Active, session_key1)
            .await
            .unwrap();

        server
            .post_update_image_add(
                post1.id,
                &[input1_img.saved_path.to_str().unwrap()],
                session_key1,
            )
            .await
            .unwrap();

        server
            .post_update_image_add(
                post1.id,
                &[input2_img.saved_path.to_str().unwrap()],
                session_key1,
            )
            .await
            .unwrap();

        let post2 = server
            .post_add("title", "description1", "tags1", session_key1)
            .await
            .unwrap();

        server
            .post_update_state(post2.id, PostState::Active, session_key1)
            .await
            .unwrap();

        server
            .post_update_image_add(
                post2.id,
                &[input2_img.saved_path.to_str().unwrap()],
                session_key1,
            )
            .await
            .unwrap();

        (post1, post2)
    };

    let _result = server.post_get_by_id(post1.id, session_key1).await.unwrap();

    // assert errors
    {
        let result = server.post_remove(post1.id, session_key2).await;
        assert!(matches!(result, Err(PostRemoveErr::Unauthorized(_))));

        let result = server.post_get_by_id(0, session_key1).await;
        assert!(matches!(result, Err(PostGetByKeyErr::PostNotFound)));
    }

    // assert success
    {
        use catsquad_db::DbFileImageGetByHashErr;

        use crate::proccess_images::proccess_images_all;

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
            .await
            .unwrap();

        assert_eq!(img1.used_count, 1);
        assert_eq!(img2.used_count, 2);
        assert!(input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());
        assert!(input2_img.storage_path.exists());
        assert!(!input2_img.thumbnail_path.exists());

        proccess_images_all(0, server.state.db.clone(), storage_path, 1280)
            .await
            .unwrap();

        assert!(input1_img.storage_path.exists());
        assert!(input1_img.thumbnail_path.exists());
        assert!(input2_img.storage_path.exists());
        assert!(input2_img.thumbnail_path.exists());

        server.post_remove(post1.id, session_key1).await.unwrap();

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
        let post1 = server.post_get_by_id(post1.id, session_key1).await;
        let post2 = server.post_get_by_id(post2.id, session_key1).await.unwrap();

        assert!(matches!(post1, Err(PostGetByKeyErr::PostNotFound)));
        assert_eq!(post2.images.len(), 1);
        assert_eq!(post2.images[0].hash, input2_img.hash);

        assert!(matches!(img1, Err(DbFileImageGetByHashErr::NotFound)));
        assert_eq!(img2.used_count, 1);

        assert!(!input1_img.storage_path.exists());
        assert!(!input1_img.thumbnail_path.exists());
        assert!(input2_img.storage_path.exists());
        assert!(input2_img.thumbnail_path.exists());
    }
}
