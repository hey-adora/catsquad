use crate::{
    api::user_update_pfp::remove_images,
    auth::{create_deleted_cookie, verify_password},
    state::AppState,
};
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbTokenUpdateConfirmErr, DbUser, DbUserRemoveByTokenErr};
use catsquad_shared::{UserRemoveByTokenErr, UserRemoveByTokenReq};

fn from_db_user_remove_by_token_err(value: DbUserRemoveByTokenErr) -> UserRemoveByTokenErr {
    match value {
        DbUserRemoveByTokenErr::Unauthorized => {
            UserRemoveByTokenErr::Unauthorized("unauthorized".to_string())
        }
        DbUserRemoveByTokenErr::Token(DbTokenUpdateConfirmErr::TokenNotFound) => {
            UserRemoveByTokenErr::TokenNotFound
        }
        DbUserRemoveByTokenErr::Token(DbTokenUpdateConfirmErr::Expired) => {
            UserRemoveByTokenErr::Expired
        }
        DbUserRemoveByTokenErr::Token(DbTokenUpdateConfirmErr::AlreadyUsed) => {
            UserRemoveByTokenErr::AlreadyUsed
        }
        DbUserRemoveByTokenErr::Token(DbTokenUpdateConfirmErr::Db(_)) => {
            UserRemoveByTokenErr::InternalServer
        }
        DbUserRemoveByTokenErr::Image(_) => UserRemoveByTokenErr::InternalServer,
        DbUserRemoveByTokenErr::Db(_) => UserRemoveByTokenErr::InternalServer,
    }
}

fn status_code(result: &Result<(), UserRemoveByTokenErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserRemoveByTokenErr::Expired) => StatusCode::BAD_REQUEST,
        Err(UserRemoveByTokenErr::AlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(UserRemoveByTokenErr::TokenNotFound) => StatusCode::BAD_REQUEST,
        Err(UserRemoveByTokenErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(UserRemoveByTokenErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserRemoveByTokenErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_remove_by_token(
    db_user: Extension<DbUser>,
    State(app): State<AppState>,
    Form(req): Form<UserRemoveByTokenReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();
    let token = req.token;
    let user_username = db_user.username.clone();
    let storage_path = app.get_storage_path().await;

    let inner = async || -> Result<(), UserRemoveByTokenErr> {
        let callback_remove_image = async move |file_hash: i64, file_extensoin: &str| {
            remove_images(storage_path.as_path(), file_hash, file_extensoin).await;
        };

        app.db
            .user_remove_by_token(time, callback_remove_image, token, user_username)
            .await
            .map_err(from_db_user_remove_by_token_err)?;

        Ok(())
    };

    let result = inner().await;
    let status_code = status_code(&result);
    let headers = create_deleted_cookie();

    (status_code, headers, Json(result))
}

#[cfg(test)]
mod test_utils {
    use axum::http::header;
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn user_remove_by_token(
            &self,
            token: Uuid,
            session_token: Uuid,
        ) -> Result<(), cs::UserRemoveByTokenErr> {
            self.client
                .user_remove_by_token(token)
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

// TODO assert if it removes all images correctly
// TODO assert if delete cookie is set correctly

#[tokio::test]
async fn test_api_user_remove_by_token() {
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_remove_by_token").await;

    let pss = "a1234567890111GG11$";
    let (_user1, user_token) = server.user_add_full("hey", "hey@heyadora.com", pss).await;
    let _ = server.user_add_full("hey2", "hey2@heyadora.com", pss).await;

    // assert success
    {
        server.token_user_remove_add(user_token).await.unwrap();
        let token = server.state.db.token_get_all().await.unwrap()[0].clone();

        server
            .user_remove_by_token(token.id, user_token)
            .await
            .unwrap();

        let users = server.state.db.token_get_all().await.unwrap();
        assert_eq!(users.len(), 0);
    }
}
