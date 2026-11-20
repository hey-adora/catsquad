use crate::{
    api::post_image_bytes_get_by_hash::get_file_image, auth::SessionTokenV2, state::AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use catsquad_shared::PostImageBytesGetByHashParams;

pub async fn post_thumbnail_bytes_get_by_hash(
    auth_session_token: SessionTokenV2,
    State(app): State<AppState>,
    Path(params): Path<PostImageBytesGetByHashParams>,
) -> impl IntoResponse {
    //TODO optimize this nonsense

    let post_id = params.post_id;
    let file_hash = params.file_hash;

    get_file_image(app, auth_session_token, post_id, file_hash, true).await
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
    use crate::promote_admin;
    use crate::{get_file_hash_for_testing_by_path, proccess_images::proccess_images_all};
    use catsquad_log::prelude::*;
    use catsquad_shared::UUID_ZERO;
    use catsquad_shared::{PostImageBytesGetByHashErr, PostState};
    use catsquad_shared::{SessionTokenKind, UserState};

    init_log();
    let server = crate::TestServer::new(0, "test_api_post_thumbnail_bytes_by_hash").await;

    let (_user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (_user2, session_key2) = server
        .user_add_full("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1.clone())
        .await
        .unwrap();

    let file_hash = get_file_hash_for_testing_by_path("../assets/favicon.ico").await;

    // assert error
    {
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
            .post_update_image_add(post1.id, &["../assets/favicon.ico"], session_key1)
            .await
            .unwrap();

        let result = server
            .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key1)
            .await;

        assert!(matches!(
            result,
            Err(PostImageBytesGetByHashErr::FileNotFound)
        ));
    }

    // assert success
    {
        proccess_images_all(
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

    // assert error unauthorized
    {
        server
            .state
            .db
            .user_update_state(
                async |_| String::new(),
                0,
                SessionTokenKind::Root,
                vec!["prime".to_string()],
                UserState::Suspended,
            )
            .await
            .unwrap();

        let result = server
            .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key1)
            .await;

        assert!(matches!(
            result,
            Err(PostImageBytesGetByHashErr::Unauthorized(_))
        ));

        let result = server
            .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key2)
            .await;

        assert!(matches!(
            result,
            Err(PostImageBytesGetByHashErr::Unauthorized(_))
        ));

        let result = server
            .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, UUID_ZERO)
            .await;

        assert!(matches!(
            result,
            Err(PostImageBytesGetByHashErr::Unauthorized(_))
        ));

        server.state.set_admin_username("prime2").await;
        promote_admin(server.state.clone()).await.unwrap();

        let result = server
            .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key2)
            .await;

        assert!(matches!(result, Ok(_)));

        let result = server
            .post_thumbnail_bytes_get_by_hash(post1.id, file_hash, session_key1)
            .await;

        assert!(matches!(
            result,
            Err(PostImageBytesGetByHashErr::Unauthorized(_))
        ));
    }
}
