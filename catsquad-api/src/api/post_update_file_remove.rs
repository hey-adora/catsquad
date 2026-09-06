use crate::{api::post_add::from_db_post, state::AppState};
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbPostUpdateFileRemoveErr, DbUser};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PostFile, PostRes, PostState, PostUpdateFileRemoveErr, PostUpdateFileRemoveReq,
};

fn from_db_post_update_file_remove_err(
    value: DbPostUpdateFileRemoveErr,
) -> PostUpdateFileRemoveErr {
    match value {
        DbPostUpdateFileRemoveErr::PostNotFound => PostUpdateFileRemoveErr::PostNotFound,
        DbPostUpdateFileRemoveErr::Unauthorized => {
            PostUpdateFileRemoveErr::Unauthorized("unauthorized".to_string())
        }
        DbPostUpdateFileRemoveErr::FileNotFound => PostUpdateFileRemoveErr::FileNotFound,
        DbPostUpdateFileRemoveErr::Db(_) => PostUpdateFileRemoveErr::InternalServer,
        DbPostUpdateFileRemoveErr::InternalError(_) => PostUpdateFileRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<(), PostUpdateFileRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(PostUpdateFileRemoveErr::PostNotFound) => StatusCode::NOT_FOUND,
        Err(PostUpdateFileRemoveErr::FileNotFound) => StatusCode::BAD_REQUEST,
        Err(PostUpdateFileRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(PostUpdateFileRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn post_update_file_remove(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<PostUpdateFileRemoveReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), PostUpdateFileRemoveErr> {
        let user_username = db_user.username.clone();
        let post_id = req.post_id;
        let hash = req.hash;

        let img_used_count = app
            .db
            .post_update_file_remove(time, user_username, post_id, hash)
            .await
            .map_err(from_db_post_update_file_remove_err)?;

        // TODO remove file on used count zero // do it in commit somehow

        // let mut post = None;
        // for file_hash in hashes {
        //     let result = app
        //         .db
        //         .post_update_file_remove(time, user_id.clone(), post_key.clone(), file_hash)
        //         .await
        //         .map_err(from_db_post_update_file_remove_err)?;
        //     post = Some(result);
        // }
        // let post = post.ok_or_else(|| PostUpdateFileRemoveErr::InternalServer)?;

        Ok(())
        // Ok(from_db_post(post))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, PostFile, PostState, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn post_update_file_remove(
            &self,
            post_id: i64,
            file_hash: i64,
            session_token: Uuid,
        ) -> Result<(), cs::PostUpdateFileRemoveErr> {
            self.client
                .post_update_file_remove(post_id, file_hash)
                .header_add(
                    header::COOKIE,
                    create_auth_cookie_str(uuid_to_str(session_token)),
                )
                .send()
                .await
                .into_json()
                .await
            // self.client
            //     .post_update_tags(post_id, new_tags)
            //     .header_add(
            //         header::COOKIE,
            //         create_auth_cookie_str(uuid_to_str(session_token)),
            //     )
            //     .send()
            //     .await
            //     .into_json()
            //     .await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_post_update_file_remove() {
    use crate::auth::create_auth_cookie_str;
    use axum::http::header;

    init_log();

    let server = crate::TestServer::new(0, "test_api_post_update_file_remove").await;

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

    let _result = server
        .post_update_file_add(post1.id, &["../assets/favicon.ico"], session_key1)
        .await
        .unwrap();

    server
        .post_update_state(post1.id, PostState::Active, session_key1)
        .await
        .unwrap();

    let result = server
        .post_get_by_key(post1.id, session_key1)
        .await
        .unwrap();

    assert_eq!(result.file.len(), 1);
    let file1_hash = result.file[0].hash;

    server
        .post_update_file_remove(post1.id, file1_hash, session_key1)
        .await
        .unwrap();

    let result = server
        .post_get_by_key(post1.id, session_key1)
        .await
        .unwrap();

    assert_eq!(result.file.len(), 0);
}
