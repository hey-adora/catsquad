use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateBuilderTextErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{PostRes, PostUpdateTagsErr, PostUpdateTagsReq, validate_post_tags};

use crate::{api::post_add::from_db_post, state::AppState};

fn from_db_post_update_tags_err(value: DbPostUpdateBuilderTextErr) -> PostUpdateTagsErr {
    match value {
        DbPostUpdateBuilderTextErr::PostNotFound => PostUpdateTagsErr::PostNotFound,
        DbPostUpdateBuilderTextErr::Unauthorized => {
            PostUpdateTagsErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateBuilderTextErr::Db(_) => PostUpdateTagsErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostUpdateTagsErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateTagsErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateTagsErr::InvalidTags(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateTagsErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateTagsErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_tags(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateTagsReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), PostUpdateTagsErr> {
        let user_username = db_user.username.clone();
        let post_id = req.post_id;
        let new_tags = req.new_tags;

        validate_post_tags(&new_tags).map_err(|err| PostUpdateTagsErr::InvalidTags(err))?;

        app.db
            .post_update_tags(time, user_username, post_id, &new_tags)
            .await
            .map_err(from_db_post_update_tags_err)?;

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
        pub async fn post_update_tags(
            &self,
            post_id: i64,
            new_tags: impl Into<String>,
            session_token: Uuid,
        ) -> Result<(), cs::PostUpdateTagsErr> {
            self.client
                .post_update_tags(post_id, new_tags)
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
async fn test_post_update_tags() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new().await;

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
        .post_update_tags(post1.id, "     tagS2", session_key1)
        .await
        .unwrap();
    let post1 = server
        .post_get_by_key(post1.id, session_key1)
        .await
        .unwrap();
    assert_eq!(post1.tags, " tags2 ");

    let result = server
        .post_update_tags(post1.id, "tags3", session_key2)
        .await;
    assert!(matches!(result, Err(PostUpdateTagsErr::Unauthorized(_))));

    let result = server.post_update_tags(0, "tags3", session_key1).await;
    assert!(matches!(result, Err(PostUpdateTagsErr::PostNotFound)));

    let result = server.post_update_tags(0, "tags3", session_key1).await;
    assert!(matches!(result, Err(PostUpdateTagsErr::PostNotFound)));
}
