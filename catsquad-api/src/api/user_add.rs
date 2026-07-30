use axum::{
    Form, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use catsquad_db::{DbSessionAddErr, DbUser, DbUserAddErr, id_to_string};
use catsquad_log::prelude::*;
use catsquad_shared::{
    MAX_STORAGE, MAX_STORAGE_PER_FILE, RedactedUserRes, SensitiveUserRes, UserAddErr, UserAddReq,
    validate_password, validate_username,
};

use crate::{
    auth::{create_auth_cookie, hash_password},
    state::AppState,
};

pub fn from_db_user_redacted(value: DbUser) -> RedactedUserRes {
    RedactedUserRes {
        key: id_to_string(value.id),
        username: value.username,
        created_at: value.created_at,
    }
}

pub fn from_db_user_sensitive(value: DbUser) -> SensitiveUserRes {
    SensitiveUserRes {
        key: id_to_string(value.id),
        username: value.username,
        email: value.email,
        created_at: value.created_at,
    }
}

fn from_db_user_add_err(value: DbUserAddErr) -> UserAddErr {
    match value {
        DbUserAddErr::EmailIsTaken => UserAddErr::EmailIsTaken,
        DbUserAddErr::UsernameIsTaken => UserAddErr::UsernameIsTaken,
        DbUserAddErr::InviteNotFound => UserAddErr::InviteNotFound,
        DbUserAddErr::InviteAlreadyUsed => UserAddErr::InviteAlreadyUsed,
        DbUserAddErr::InviteExpired => UserAddErr::InviteExpired,
        DbUserAddErr::Db(_) => UserAddErr::InternalServer,
    }
}

fn from_db_session_add_err(value: DbSessionAddErr) -> UserAddErr {
    match value {
        _ => UserAddErr::InternalServer,
    }
}

fn from_argon_err(value: argon2::password_hash::Error) -> UserAddErr {
    match value {
        _ => UserAddErr::InternalServer,
    }
}

fn status_code(result: &Result<SensitiveUserRes, UserAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserAddErr::InvalidInput { .. }) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::EmailIsTaken) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::UsernameIsTaken) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::InviteNotFound) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::InviteAlreadyUsed) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::InviteExpired) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(UserAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_add(
    State(app): State<AppState>,
    Form(req): Form<UserAddReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let inner = async || -> Result<SensitiveUserRes, UserAddErr> {
        let username = req.username.trim().to_lowercase();
        let password = req.password;
        let username_err = validate_username(&username);
        let password_err = validate_password(&password);

        if username_err.is_err() || password_err.is_err() {
            return Err(UserAddErr::InvalidInput {
                username: username_err.err(),
                password: password_err.err(),
            });
        }

        let password = hash_password(password)
            .inspect_err(|err| error!("{err}"))
            .map_err(from_argon_err)?;

        let user = app
            .db
            .user_add(
                time,
                username,
                password,
                req.invite_key,
                MAX_STORAGE,
                MAX_STORAGE_PER_FILE,
            )
            .await
            .map_err(from_db_user_add_err)?;

        let user = from_db_user_sensitive(user);
        Ok(user)
    };
    let user_add_result = inner().await;
    match user_add_result {
        Ok(result) => {
            let session_add_result = app
                .db
                .session_add(time, &result.email)
                .await
                .map_err(from_db_session_add_err);

            let headers = match session_add_result {
                Ok(session) => create_auth_cookie(id_to_string(session.id)),
                Err(_) => {
                    let headers = HeaderMap::new();
                    let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                    let result = Err(UserAddErr::InternalServer);
                    return (status_code, headers, Json(result));
                }
            };

            let result = Ok(result);
            let status_code = status_code(&result);
            (status_code, headers, Json(result))
        }

        Err(err) => {
            let result = Err(err);
            let status_code = status_code(&result);
            let headers = HeaderMap::new();
            (status_code, headers, Json(result))
        }
    }
}

#[cfg(any(test, feature = "test_server"))]
mod test_utils {
    use axum::http::{
        HeaderName,
        header::{self, SET_COOKIE},
    };
    use catsquad_db::{DbUser, id_to_string};
    use catsquad_shared::{LINK_API_USER_ADD, ToForm, UserAddReq};

    use crate::{
        TestServer,
        api::user_add::{SensitiveUserRes, UserAddErr},
        auth::auth_token_get,
    };

    impl TestServer {
        // pub async fn user_add(
        //     &self,
        //     username: impl Into<String>,
        //     password: impl Into<String>,
        //     invite_token: impl Into<String>,
        // ) -> (Result<UserRes, UserAddErr>, Option<String>) {
        //     let data = UserAddReq {
        //         username: username.into(),
        //         password: password.into(),
        //         invite_key: invite_token.into(),
        //     }
        //     .to_form()
        //     .unwrap();
        //     self.post_and_get_auth_token::<Result<UserRes, UserAddErr>>(LINK_API_USER_ADD, data)
        //         .await
        // }

        pub async fn user_add(
            &self,
            user: impl Into<String>,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> (DbUser, String) {
            let username = user.into();
            let email = email.into();
            let password = password.into();
            let _invite = self
                .client
                .invite_add(email.clone())
                .await
                .send()
                .await
                .into_res()
                .await
                .unwrap();
            // let invite = self.invite_add(email.clone()).await.unwrap();
            let invite = id_to_string(
                self.state
                    .db
                    .invite_get_all()
                    .await
                    .unwrap()
                    .into_iter()
                    .find(|v| !v.used && v.email == email)
                    .unwrap()
                    .id
                    .clone(),
            );
            let res = self
                .client
                .user_add(username, invite, password)
                .await
                .send()
                .await;
            let headers = res.get_headers().unwrap();
            let session_key = auth_token_get(&headers, header::SET_COOKIE).unwrap();
            let _res = res.into_res().await.unwrap();

            // let session_key = res.get_auth_token().unwrap();
            // let res = res.into_res().await;
            let user = self.state.db.user_get_by_email(email).await.unwrap();
            (user, session_key)
        }
    }
}

// #[tokio::test]
// async fn test_user_add() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let (result, _) = server.user_add("hey", "hello", "").await;
//     assert!(
//         matches!(result, Err(UserAddErr::InvalidInput { username, password }) if username.is_none() && password.is_some())
//     );

//     let (result, _) = server.user_add("he", "hello@P", "").await;
//     assert!(
//         matches!(result, Err(UserAddErr::InvalidInput { username, password }) if username.is_some() && password.is_some())
//     );

//     let (result, _) = server.user_add("hey", "hello1111111@1P", "invalid").await;
//     assert!(matches!(result, Err(UserAddErr::InviteNotFound)));

//     let _invite = server.invite_add("prime@heyadora.com").await.unwrap();
//     let invite = id_to_string(
//         server.state.db.invite_get_all().await.unwrap()[0]
//             .id
//             .clone(),
//     );
//     let (result, token) = server.user_add("hey", "hello1111111@1P", invite).await;
//     assert!(matches!(result, Ok(_)));
//     let _result = server
//         .state
//         .db
//         .session_get_by_key(token.unwrap())
//         .await
//         .unwrap();

//     // check if its encrypted
//     let user = server.state.db.user_get_by_username("hey").await.unwrap();
//     assert_ne!(user.password, "hello1111111@1P");
// }
