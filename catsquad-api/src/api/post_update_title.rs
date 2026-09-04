use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateBuilderTextErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostRes, PostUpdateTitleErr, PostUpdateTitleReq, validate_post_title};

use crate::{api::post_add::from_db_post, auth::verify_password, state::AppState};

fn from_db_post_update_title_err(value: DbPostUpdateBuilderTextErr) -> PostUpdateTitleErr {
    match value {
        DbPostUpdateBuilderTextErr::PostNotFound => PostUpdateTitleErr::PostNotFound,
        DbPostUpdateBuilderTextErr::Unauthorized => {
            PostUpdateTitleErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateBuilderTextErr::Db(_) => PostUpdateTitleErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostUpdateTitleErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateTitleErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateTitleErr::InvalidTitle(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateTitleErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateTitleErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_title(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateTitleReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), PostUpdateTitleErr> {
        let user_id = db_user.username.clone();
        let post_key = req.post_id;
        let new_title = req.new_title;

        validate_post_title(&new_title).map_err(|err| PostUpdateTitleErr::InvalidTitle(err))?;

        let result = app
            .db
            .post_update_title(time, user_id, post_key, &new_title)
            .await
            .map_err(from_db_post_update_title_err)?;

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
    use catsquad_shared::{self as cs, PostState, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn post_update_title(
            &self,
            post_id: i64,
            new_title: impl Into<String>,
            session_token: Uuid,
        ) -> Result<(), cs::PostUpdateTitleErr> {
            self.client
                .post_update_title(post_id, new_title)
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
async fn test_post_update_title() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new().await;

    let (user1, session_key1) = server
        .user_add_full("prime", "prime@heyadora.com", "1234567890111GGd11$")
        .await;

    let (user2, session_key2) = server
        .user_add_full("prime2", "prime2@heyadora.com", "1234567890111GGd11$")
        .await;

    let post1 = server
        .post_add("title", "description1", "tags1", session_key1)
        .await
        .unwrap();

    server
        .post_update_title(post1.id, "title2", session_key1)
        .await
        .unwrap();
    let post1 = server
        .post_get_by_key(post1.id, session_key1)
        .await
        .unwrap();

    assert_eq!(post1.title, "title2");

    let result = server
        .post_update_title(post1.id, "title3", session_key2)
        .await;
    assert!(matches!(result, Err(PostUpdateTitleErr::Unauthorized(_))));

    let result = server.post_update_title(0, "title3", session_key1).await;
    assert!(matches!(result, Err(PostUpdateTitleErr::PostNotFound)));

    let result = server.post_update_title(0, "title3", session_key1).await;
    assert!(matches!(result, Err(PostUpdateTitleErr::PostNotFound)));
}
