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
    use catsquad_shared as cs;

    use crate::{
        TestServer,
        api::user_add::{SensitiveUserRes, UserAddErr},
        auth::{auth_token_get, create_auth_cookie_str},
    };

    impl TestServer {
        pub async fn user_add(
            &self,
            username: impl Into<String>,
            invite_key: impl Into<String>,
            password: impl Into<String>,
        ) -> Result<cs::SensitiveUserRes, cs::UserAddErr> {
            self.client
                .user_add(username, invite_key, password)
                .send()
                .await
                .into_res()
                .await
        }
        pub async fn user_add_with_session(
            &self,
            username: impl Into<String>,
            invite_key: impl Into<String>,
            password: impl Into<String>,
        ) -> (cs::SensitiveUserRes, String) {
            let res = self
                .client
                .user_add(username, invite_key, password)
                .send()
                .await;
            let headers = res.get_headers().unwrap();
            let session_key = auth_token_get(&headers, header::SET_COOKIE).unwrap();
            let result = res.into_res().await.unwrap();
            (result, session_key)
        }
        pub async fn user_add_full(
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

// #[cfg(test)]
// mod test_utils {
//     use crate::{TestServer, auth::create_auth_cookie_str};
//     use axum::http::header;
//     use catsquad_shared as cs;

//     impl TestServer {
//         pub async fn email_change_update_cancel(
//             &self,
//             email_change_key: impl Into<String>,
//             session_key: impl Into<String>,
//         ) -> Result<cs::EmailChangeRes, cs::EmailChangeUpdateCancelErr> {
//             self.client
//                 .email_change_update_cancel(email_change_key)
//                 .header_add(header::COOKIE, create_auth_cookie_str(session_key.into()))
//                 .send()
//                 .await
//                 .into_res()
//                 .await
//         }
//     }
// }

#[cfg(test)]
#[tokio::test]
async fn test_user_add() {
    init_log();
    let server = crate::TestServer::new().await;

    // invalid input
    {
        let result = server.user_add("hey", "hello", "").await;
        assert!(
            matches!(result, Err(UserAddErr::InvalidInput { username, password }) if username.is_none() && password.is_some())
        );

        let result = server.user_add("he", "hello@P", "").await;
        assert!(
            matches!(result, Err(UserAddErr::InvalidInput { username, password }) if username.is_some() && password.is_some())
        );
    }

    // invite not found
    {
        let result = server
            .user_add("hey", "hello1111111@1P", "inesognf042n0NR)TN09nnfw9")
            .await;
        assert!(matches!(result, Err(UserAddErr::InviteNotFound)));
    }

    // check if creates session
    {
        let _invite = server.invite_add("prime@heyadora.com").await.unwrap();
        let invite_key = server.invite_get_key("prime@heyadora.com").await;
        let (_result, session_key) = server
            .user_add_with_session("prime", invite_key, "inesognf042n0NR)TN09nnfw9")
            .await;
        let _result = server
            .state
            .db
            .session_get_by_key(session_key)
            .await
            .unwrap();
    }

    // check if encrypted
    {
        let user = server.state.db.user_get_by_username("prime").await.unwrap();
        assert_ne!(user.password, "hello1111111@1P");
    }
}
