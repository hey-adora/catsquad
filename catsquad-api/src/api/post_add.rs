use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPost, DbPostAddErr, DbPostFile, DbUser, id_to_string};
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
        key: id_to_string(value.id),
        user: from_db_user_redacted(value.user),
        state: PostState::from(value.state),
        title: value.title,
        tags: value.tags,
        favorites: value.favorites,
        description: value.description,
        file: value.file.into_iter().map(from_db_post_file).collect(),
        modified_at: value.modified_at,
        created_at: value.created_at,
    }
}

pub fn from_db_post_file(value: DbPostFile) -> PostFile {
    PostFile {
        extension: value.extension,
        hash: value.hash,
        proccesed: value.proccesed,
        size_bytes: value.size_bytes,
        width: value.width,
        height: value.height,
    }
}

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
    let time = app.get_time().await;
    let inner = async || -> Result<PostRes, PostAddErr> {
        let title = req.title.trim();
        let description = req.description.trim();
        let tags = req.tags.trim();
        let user_id = db_user.id.clone();

        validate_post_title(title).map_err(|err| PostAddErr::InvalidTitle(err))?;
        validate_post_tags(tags).map_err(|err| PostAddErr::InvalidTags(err))?;
        validate_post_description(description)
            .map_err(|err| PostAddErr::InvalidDescription(err))?;

        let post = app
            .db
            .post_add(time, user_id, title, description, tags)
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
    use catsquad_shared as cs;

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn post_add(
            &self,
            title: impl Into<String>,
            description: impl Into<String>,
            tags: impl Into<String>,
            session_key: impl Into<String>,
        ) -> Result<cs::PostRes, cs::PostAddErr> {
            self.client
                .post_add(title, description, tags)
                .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
                .send()
                .await
                .into_json()
                .await
        }
    }
}

#[tokio::test]
async fn test_post_add() {
    init_log();
    let server = crate::TestServer::new().await;

    let email = "hey@heyadora.com";
    let password = "1nnerogGeron@@$";
    let (user, session_key) = server.user_add_full("hey", email, password).await;

    server.state.set_time(1).await;

    let post1 = server
        .post_add("title1", "description1", "tags1", &session_key)
        .await
        .unwrap();

    assert_eq!(post1.created_at, 1);

    let post1 = server
        .post_update_state(post1.key.clone(), PostState::Active, &session_key)
        .await
        .unwrap();

    assert_eq!(post1.created_at, 1);

    server.state.set_time(2).await;

    let post2 = server
        .post_add("title2", "description2", "tags2", &session_key)
        .await
        .unwrap();

    assert_eq!(post2.created_at, 2);

    let post2 = server
        .post_update_state(post2.key.clone(), PostState::Active, &session_key)
        .await
        .unwrap();

    assert_eq!(post2.created_at, 2);
}
