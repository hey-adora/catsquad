use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use catsquad_db::DbUser;
use catsquad_shared::PostImageBytesGetByHashParams;

use crate::{api::post_file_bytes_get_by_hash::get_file_image, state::AppState};

pub async fn post_thumbnail_bytes_get_by_hash(
    db_user: Extension<Option<DbUser>>,
    State(app): State<AppState>,
    Path(params): Path<PostImageBytesGetByHashParams>,
) -> impl IntoResponse {
    //TODO optimize this nonsense

    let user_username = db_user
        .as_ref()
        .map(|v| v.username.clone())
        .unwrap_or_default();
    let post_id = params.post_id;
    let file_hash = params.file_hash;

    get_file_image(app, user_username, post_id, file_hash, true).await
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn post_thumbnail_bytes_get_by_hash(
            &self,
            post_id: i64,
            file_hash: i64,
            session_token: Uuid,
        ) -> Result<Vec<u8>, cs::PostImageBytesGetByHashErr> {
            self.client
                .post_thumbnail_bytes_get_by_hash(post_id, file_hash)
                .header_add(
                    header::COOKIE,
                    create_auth_cookie_str(uuid_to_str(session_token)),
                )
                .send()
                .await
                .into_bytes()
                .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_post_thumbnail_bytes_by_hash() {
    use crate::{get_file_hash_for_testing_by_path, proccess_images::proccess_post_images};
    use catsquad_log::prelude::*;
    use catsquad_shared::{PostImageBytesGetByHashErr, PostState};
    use tokio::fs::set_permissions;

    init_log();
    let server = crate::TestServer::new(0, "test_api_post_file_by_hash").await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1.clone())
        .await
        .unwrap();

    let file_hash = get_file_hash_for_testing_by_path("../assets/favicon.ico").await;

    let result = server
        .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key1)
        .await;
    assert!(matches!(
        result,
        Err(PostImageBytesGetByHashErr::FileNotFound)
    ));

    server
        .post_update_state(post1.id, PostState::Active, session_key1)
        .await
        .unwrap();

    let result = server
        .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key1)
        .await;
    assert!(matches!(
        result,
        Err(PostImageBytesGetByHashErr::FileNotFound)
    ));

    let _result = server
        .post_update_file_add(post1.id, &["../assets/favicon.ico"], session_key1)
        .await
        .unwrap();

    let result = server
        .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key1)
        .await;

    assert!(matches!(
        result,
        Err(PostImageBytesGetByHashErr::FileNotFound)
    ));

    proccess_post_images(
        0,
        server.state.db.clone(),
        server.state.get_storage_path().await,
        1280,
    )
    .await
    .unwrap();

    let result = server
        .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key1)
        .await
        .unwrap();
    assert!(result.len() > 0);
}
