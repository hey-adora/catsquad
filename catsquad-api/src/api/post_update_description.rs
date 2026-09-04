use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateBuilderTextErr, DbUser};
use catsquad_shared::{
    PostRes, PostUpdateDescriptionErr, PostUpdateDescriptionReq, validate_post_description,
};

use crate::{api::post_add::from_db_post, state::AppState};

fn from_db_post_update_description_err(
    value: DbPostUpdateBuilderTextErr,
) -> PostUpdateDescriptionErr {
    match value {
        DbPostUpdateBuilderTextErr::PostNotFound => PostUpdateDescriptionErr::PostNotFound,
        DbPostUpdateBuilderTextErr::Unauthorized => {
            PostUpdateDescriptionErr::Unauthorized("unauthorized".to_string())
        }
        // DbPostUpdateDescriptionErr::UserNotFound => PostUpdateDescriptionErr::InternalServer,
        DbPostUpdateBuilderTextErr::Db(_) => PostUpdateDescriptionErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostUpdateDescriptionErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateDescriptionErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateDescriptionErr::InvalidDescription(_)) => StatusCode::BAD_REQUEST,
        Err(PostUpdateDescriptionErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateDescriptionErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_description(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateDescriptionReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), PostUpdateDescriptionErr> {
        let user_username = db_user.username.clone();
        let post_id = req.post_id;
        let new_description = req.new_description;

        validate_post_description(&new_description)
            .map_err(|err| PostUpdateDescriptionErr::InvalidDescription(err))?;

        app.db
            .post_update_description(time, user_username, post_id, &new_description)
            .await
            .map_err(from_db_post_update_description_err)?;

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
        pub async fn post_update_description(
            &self,
            post_id: i64,
            new_description: impl Into<String>,
            session_token: Uuid,
        ) -> Result<(), cs::PostUpdateDescriptionErr> {
            self.client
                .post_update_description(post_id, new_description)
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
async fn test_post_update_description() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;
    use catsquad_log::prelude::*;

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
        .post_update_description(post1.id, "description2", session_key1)
        .await
        .unwrap();
    let post1 = server
        .post_get_by_key(post1.id, session_key1)
        .await
        .unwrap();

    assert_eq!(post1.description, "description2");

    let result = server
        .post_update_description(post1.id, "description3", session_key2)
        .await;
    assert!(matches!(
        result,
        Err(PostUpdateDescriptionErr::Unauthorized(_))
    ));

    let result = server
        .post_update_description(0, "title3", session_key1)
        .await;
    assert!(matches!(
        result,
        Err(PostUpdateDescriptionErr::PostNotFound)
    ));

    let result = server
        .post_update_description(0, "title3", session_key1)
        .await;
    assert!(matches!(
        result,
        Err(PostUpdateDescriptionErr::PostNotFound)
    ));
}
