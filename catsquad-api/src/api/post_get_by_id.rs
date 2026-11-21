use crate::{auth::SessionTokenV2, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbFileImage, DbPostGet, DbPostGetByKeyErr};
use catsquad_log::prelude::*;
use catsquad_shared::{FileImage, PostGetByKeyErr, PostGetByKeyParams, PostRes, PostState};

pub fn from_db_post_get_by_key_err(value: DbPostGetByKeyErr) -> PostGetByKeyErr {
    match value {
        DbPostGetByKeyErr::Unauthorized(err) => PostGetByKeyErr::Unauthorized(err),
        DbPostGetByKeyErr::PostNotFound => PostGetByKeyErr::PostNotFound,
        DbPostGetByKeyErr::Db(_) => PostGetByKeyErr::InternalServerErr,
        DbPostGetByKeyErr::InternalServerErr => PostGetByKeyErr::InternalServerErr,
    }
}

pub fn from_db_post_get(value: DbPostGet) -> PostRes {
    PostRes {
        id: value.id,
        user_username: value.user_username,
        user_state: value.user_state,
        user_pfp_hash: value.user_pfp_hash,
        user_pfp_proccesed: value.user_pfp_proccesed,
        state: PostState::try_from(value.state)
            .inspect_err(|err| error!("{err}"))
            .unwrap_or(PostState::Active),
        title: value.title,
        tags: value.tags,
        followers: value.followers,
        followed: value.followed,
        liked: value.liked,
        likes_count: value.likes_count,
        description: value.description,
        images: value.images.into_iter().map(from_db_post_file).collect(),
        modified_at: value.modified_at,
        created_at: value.created_at,
    }
}

pub fn from_db_post_file(value: DbFileImage) -> FileImage {
    FileImage {
        extension: value.extension,
        hash: value.hash,
        proccesed: value.processed,
        size_bytes: value.size_bytes,
        width: value.width,
        height: value.height,
    }
}

// fn params_req(value: RawPathParams) -> Result<PostGetByKeyParams, PostGetByKeyErr> {
//     value
//         .iter()
//         .find(|(name, _)| *name == POST_GET_BY_KEY_REQ_FIELD_POST_KEY)
//         .ok_or(PostGetByKeyErr::BadRequest(
//             "missing post_key param".to_string(),
//         ))
//         .map(|(_, value)| PostGetByKeyParams {
//             post_key: value.to_string(),
//         })
// }

pub fn status_code(result: &Result<PostRes, PostGetByKeyErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostGetByKeyErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostGetByKeyErr::PostNotFound) => StatusCode::NOT_FOUND,
        // Err(PostGetByKeyErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(PostGetByKeyErr::InternalServerErr) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_get_by_id(
    auth_session_token: SessionTokenV2,
    State(app): State<AppState>,
    Path(params): Path<PostGetByKeyParams>,
) -> impl IntoResponse {
    let inner = async || -> Result<PostRes, PostGetByKeyErr> {
        let post = app
            .db
            .post_get_by_id(auth_session_token.into(), params.post_id)
            .await
            .map_err(from_db_post_get_by_key_err)?;

        Ok(from_db_post_get(post))
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
        pub async fn post_get_by_id(
            &self,
            post_id: i64,
            session_token: Uuid,
        ) -> Result<cs::PostRes, cs::PostGetByKeyErr> {
            self.client
                .post_get_by_key(post_id)
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
async fn test_api_post_get_by_key() {
    use crate::promote_admin;
    use catsquad_shared::{SessionTokenKind, UUID_ZERO, UserState};

    init_log();
    let server = crate::TestServer::new(0, "test_api_post_get_by_key").await;

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

    // assert error
    {
        let result = server.post_get_by_id(0, 0_u128.to_be_bytes()).await;
        assert_eq!(result, Err(PostGetByKeyErr::PostNotFound));

        let result = server.post_get_by_id(post1.id, 0_u128.to_be_bytes()).await;
        assert!(matches!(result, Err(PostGetByKeyErr::PostNotFound)));

        let result = server.post_get_by_id(post1.id, session_key1).await;
        assert!(matches!(result, Err(PostGetByKeyErr::PostNotFound)));

        let result = server.post_get_by_id(post1.id, session_key2).await;
        assert!(matches!(result, Err(PostGetByKeyErr::PostNotFound)));
    }

    let post_state = async |state: PostState| {
        server
            .state
            .db
            .post_update_state(0, "prime".to_string(), post1.id.clone(), state)
            .await
            .unwrap();
    };

    let user_state = async |user_state: UserState| {
        server
            .state
            .db
            .user_update_state(
                async |_| String::new(),
                0,
                SessionTokenKind::Root,
                vec!["prime".to_string()],
                user_state,
            )
            .await
            .unwrap();
    };

    // assert success
    {
        post_state(PostState::Active).await;

        let result = server
            .post_get_by_id(post1.id, 0_u128.to_be_bytes())
            .await
            .unwrap();
        assert_eq!(result.id, post1.id);
    }

    // assert - post hidden - user active
    {
        post_state(PostState::Hidden).await;

        let result = server.post_get_by_id(post1.id.clone(), session_key1).await;
        assert!(matches!(result, Ok(_)));

        let result = server.post_get_by_id(post1.id, session_key2).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));

        let result = server.post_get_by_id(post1.id.clone(), UUID_ZERO).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));
    }

    // assert - post active - user suspended
    {
        post_state(PostState::Active).await;
        user_state(UserState::Suspended).await;

        let result = server.post_get_by_id(post1.id.clone(), session_key1).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));

        let result = server.post_get_by_id(post1.id, session_key2).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));

        let result = server.post_get_by_id(post1.id, UUID_ZERO).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));
    }

    // assert - post hidden - user suspended
    {
        post_state(PostState::Hidden).await;
        user_state(UserState::Suspended).await;

        let result = server.post_get_by_id(post1.id.clone(), session_key1).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));

        let result = server.post_get_by_id(post1.id, session_key2).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));

        let result = server.post_get_by_id(post1.id, UUID_ZERO).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));
    }

    // assert - post hidden - user suspended -- perms see hidden
    {
        server.state.set_admin_username("prime2").await;
        promote_admin(server.state.clone()).await.unwrap();

        let result = server.post_get_by_id(post1.id.clone(), session_key1).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));

        let result = server.post_get_by_id(post1.id, session_key2).await;
        assert!(matches!(result, Ok(_)));

        let result = server.post_get_by_id(post1.id, UUID_ZERO).await;
        assert!(matches!(result, Err(PostGetByKeyErr::Unauthorized(_))));
    }
}
