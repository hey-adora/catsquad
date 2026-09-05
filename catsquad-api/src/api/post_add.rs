use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPost, DbPostAddErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PostAddErr, PostAddReq, PostFile, PostRes, PostState, validate_post_description,
    validate_post_tags, validate_post_title,
};

use crate::{
    api::user_add::{from_db_user_redacted, from_db_user_sensitive},
    state::AppState,
};

pub fn from_db_post(value: DbPost) -> PostRes {
    PostRes {
        id: value.id,
        user_username: value.user_username,
        state: PostState::from(value.state),
        title: value.title,
        tags: value.tags,
        favorites: value.likes_count,
        description: value.description,
        file: value.images_hashes,
        modified_at: value.modified_at,
        created_at: value.created_at,
    }
}

// pub fn from_db_post_file(value: DbPostFile) -> PostFile {
//     PostFile {
//         extension: value.extension,
//         hash: value.hash,
//         proccesed: value.proccesed,
//         size_bytes: value.size_bytes,
//         width: value.width,
//         height: value.height,
//     }
// }

fn from_db_post_add_err(value: DbPostAddErr) -> PostAddErr {
    match value {
        DbPostAddErr::UserNotFound => PostAddErr::InternalServer,
        DbPostAddErr::Db(_) => PostAddErr::InternalServer,
    }
}

fn status_code(result: &Result<PostRes, PostAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostAddErr::InvalidDescription(_)) => StatusCode::BAD_REQUEST,
        Err(PostAddErr::InvalidTitle(_)) => StatusCode::BAD_REQUEST,
        Err(PostAddErr::InvalidTags(_)) => StatusCode::BAD_REQUEST,
        Err(PostAddErr::ServerFSErr(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(PostAddErr::ServerDirCreationFailed(_)) => StatusCode::INTERNAL_SERVER_ERROR,
        Err(PostAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_add(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostAddReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let inner = async || -> Result<PostRes, PostAddErr> {
        let title = req.title.trim();
        let description = req.description.trim();
        let tags = req.tags.trim();
        let user_username = db_user.username.clone();

        validate_post_title(title).map_err(|err| PostAddErr::InvalidTitle(err))?;
        validate_post_tags(tags).map_err(|err| PostAddErr::InvalidTags(err))?;
        validate_post_description(description)
            .map_err(|err| PostAddErr::InvalidDescription(err))?;

        let post = app
            .db
            .post_add(time, user_username, title, description, tags)
            .await
            .map_err(from_db_post_add_err)?;

        Ok(from_db_post(post))
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
        pub async fn post_add(
            &self,
            title: impl Into<String>,
            description: impl Into<String>,
            tags: impl Into<String>,
            session_token: Uuid,
        ) -> Result<cs::PostRes, cs::PostAddErr> {
            self.client
                .post_add(title, description, tags)
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

#[tokio::test]
async fn test_api_post_add() {
    init_log();
    let server = crate::TestServer::new(0, "test_api_post_add").await;

    let email = "hey@heyadora.com";
    let password = "1nnerogGeron@@$";
    let (user, session_key) = server.user_add_full("hey", email, password).await;

    server.state.set_time(1);

    let post1 = server
        .post_add("title1", "description1", "tags1", session_key)
        .await
        .unwrap();

    assert_eq!(post1.created_at, 1);

    server
        .post_update_state(post1.id, PostState::Active, session_key)
        .await
        .unwrap();
    let post1 = server.post_get_by_key(post1.id, session_key).await.unwrap();
    assert_eq!(post1.created_at, 1);

    server.state.set_time(2);

    let post2 = server
        .post_add("title2", "description2", "tags2", session_key)
        .await
        .unwrap();

    assert_eq!(post2.created_at, 2);

    server
        .post_update_state(post2.id, PostState::Active, session_key)
        .await
        .unwrap();
    let post1 = server.post_get_by_key(post1.id, session_key).await.unwrap();

    assert_eq!(post2.created_at, 2);
}
