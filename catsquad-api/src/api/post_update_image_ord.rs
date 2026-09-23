use crate::state::AppState;
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateOrderErr, DbUser};
use catsquad_shared::{PostUpdateImageOrdErr, PostUpdateImageOrdReq};

fn from_db_post_update_image_ord_err(value: DbPostUpdateOrderErr) -> PostUpdateImageOrdErr {
    match value {
        DbPostUpdateOrderErr::PostNotFound => PostUpdateImageOrdErr::PostNotFound,
        DbPostUpdateOrderErr::InvalidIndex => PostUpdateImageOrdErr::InvalidIndex,
        DbPostUpdateOrderErr::Unauthorized => {
            PostUpdateImageOrdErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateOrderErr::Db(_) => PostUpdateImageOrdErr::InternalServer,
        DbPostUpdateOrderErr::InternalError(_) => PostUpdateImageOrdErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostUpdateImageOrdErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateImageOrdErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateImageOrdErr::InvalidIndex) => StatusCode::BAD_REQUEST,
        Err(PostUpdateImageOrdErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateImageOrdErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_image_ord(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateImageOrdReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), PostUpdateImageOrdErr> {
        let user_username = db_user.username.clone();
        let post_id = req.post_id;
        let pos_a = req.image_index_a;
        let pos_b = req.image_index_b;

        app.db
            .post_update_order(time, user_username, post_id, pos_a, pos_b)
            .await
            .map_err(from_db_post_update_image_ord_err)?;

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
        pub async fn post_update_file_ord(
            &self,
            post_id: i64,
            pos_a: usize,
            pos_b: usize,
            sesson_token: Uuid,
        ) -> Result<(), cs::PostUpdateImageOrdErr> {
            self.client
                .post_update_image_ord(post_id, pos_a, pos_b)
                .header_add(
                    header::COOKIE,
                    create_auth_cookie_str(uuid_to_str(sesson_token)),
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
async fn test_api_post_update_image_ord() {
    use crate::get_file_hash_for_testing_by_path;
    use catsquad_log::prelude::*;
    use catsquad_seed::create_img;

    init_log();

    let server = crate::TestServer::new(0, "test_api_post_update_image_ord").await;

    let tmp_path = server.state.get_tmp_path().await;
    let img1_path = tmp_path.join("img1.png");
    let img1_path_str = img1_path.to_str().unwrap();
    let img2_path = tmp_path.join("img2.png");
    let img2_path_str = img2_path.to_str().unwrap();
    let img3_path = tmp_path.join("img3.png");
    let img3_path_str = img3_path.to_str().unwrap();
    create_img(img1_path_str, "1");
    create_img(img2_path_str, "2");
    create_img(img3_path_str, "3");
    let img1_hash = get_file_hash_for_testing_by_path(img1_path_str).await;
    let img2_hash = get_file_hash_for_testing_by_path(img2_path_str).await;
    let img3_hash = get_file_hash_for_testing_by_path(img3_path_str).await;

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

    let _img1 = server
        .post_update_image_add(post1.id, &[img1_path_str], session_key1)
        .await
        .unwrap();

    let _img2 = server
        .post_update_image_add(post1.id, &[img2_path_str], session_key1)
        .await
        .unwrap();

    let _img3 = server
        .post_update_image_add(post1.id, &[img3_path_str], session_key1)
        .await
        .unwrap();

    // gets the current post
    let pos = server.post_add("", "", "", session_key1).await.unwrap();

    assert_eq!(
        pos.images.into_iter().map(|v| v.hash).collect::<Vec<i64>>(),
        vec![img1_hash, img2_hash, img3_hash]
    );

    server
        .post_update_file_ord(post1.id, 0, 1, session_key1)
        .await
        .unwrap();

    // gets the current post
    let pos = server.post_add("", "", "", session_key1).await.unwrap();
    assert_eq!(
        pos.images.into_iter().map(|v| v.hash).collect::<Vec<i64>>(),
        vec![img2_hash, img1_hash, img3_hash]
    );

    let result = server.post_update_file_ord(0, 0, 1, session_key1).await;
    assert!(matches!(result, Err(PostUpdateImageOrdErr::PostNotFound)));

    let result = server
        .post_update_file_ord(post1.id, 4, 1, session_key1)
        .await;
    assert!(matches!(result, Err(PostUpdateImageOrdErr::InvalidIndex)));

    let result = server
        .post_update_file_ord(post1.id, 0, 1, session_key2)
        .await;
    assert!(matches!(
        result,
        Err(PostUpdateImageOrdErr::Unauthorized(_))
    ));
}
