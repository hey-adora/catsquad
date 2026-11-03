use crate::{
    api::user_update_pfp::remove_images, auth::create_deleted_cookie, promote_admin,
    state::AppState,
};
use axum::{Extension, Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbUserGet, DbUserRemoveByUsernamesErr};
use catsquad_shared::{
    ROLE_VALUE_PERMISSION_USER_REMOVE_BY_USERNAMES, RoleValue, UserRemoveByUsernamesErr,
    UserRemoveByUsernamesReq, contains_roles,
};

fn from_db_user_remove_by_usernames_err(
    value: DbUserRemoveByUsernamesErr,
) -> UserRemoveByUsernamesErr {
    match value {
        DbUserRemoveByUsernamesErr::Unauthorized => {
            UserRemoveByUsernamesErr::Unauthorized("unauthorized".to_string())
        }
        DbUserRemoveByUsernamesErr::Image(_) => UserRemoveByUsernamesErr::InternalServer,
        DbUserRemoveByUsernamesErr::Db(_) => UserRemoveByUsernamesErr::InternalServer,
    }
}

fn status_code(result: &Result<(), UserRemoveByUsernamesErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserRemoveByUsernamesErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserRemoveByUsernamesErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_remove_by_usernames(
    db_user: Extension<DbUserGet>,
    State(app): State<AppState>,
    Form(req): Form<UserRemoveByUsernamesReq>,
) -> impl IntoResponse {
    let inner = async || -> Result<(), UserRemoveByUsernamesErr> {
        let has_permission = contains_roles(
            &[RoleValue::PermissionUserRemoveByUsername],
            &db_user.roles_values,
        );

        if !has_permission {
            return Err(UserRemoveByUsernamesErr::Unauthorized(format!(
                "user {} doesnt have permission {}",
                db_user.username, ROLE_VALUE_PERMISSION_USER_REMOVE_BY_USERNAMES
            )));
        }

        let time = app.get_time_micro();
        let usernames = req.usernames;
        let storage_path = app.get_storage_path().await;

        let callback_remove_image = async move |file_hash: i64, file_extensoin: &str| {
            remove_images(storage_path.as_path(), file_hash, file_extensoin).await;
        };

        app.db
            .user_remove_by_usernames(time, callback_remove_image, usernames)
            .await
            .map_err(from_db_user_remove_by_usernames_err)?;

        Ok(())
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
        pub async fn user_remove_by_usernames(
            &self,
            usernames: Vec<String>,
            session_token: Uuid,
        ) -> Result<(), cs::UserRemoveByUsernamesErr> {
            self.client
                .user_remove_by_usernames(usernames)
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
async fn test_api_user_remove_by_usernames() {
    use catsquad_log::prelude::*;

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_remove_by_usernames").await;

    let pss = "a1234567890111GG11$";
    let (_user0, user0_token) = server.user_add_full("hey0", "hey0@heyadora.com", pss).await;
    let (_user1, user1_token) = server.user_add_full("hey1", "hey1@heyadora.com", pss).await;
    let (_user2, _user2_token) = server.user_add_full("hey2", "hey2@heyadora.com", pss).await;

    // assert error
    {
        let result = server
            .user_remove_by_usernames(vec!["hey1".to_string(), "hey2".to_string()], user1_token)
            .await;

        assert!(matches!(
            result,
            Err(UserRemoveByUsernamesErr::Unauthorized(_))
        ));
    }

    // assert success
    {
        server.state.set_admin_username("hey0").await;
        promote_admin(server.state.clone()).await.unwrap();

        let users = server.state.db.user_get_all().await.unwrap();
        assert_eq!(users.len(), 3);

        server
            .user_remove_by_usernames(vec!["hey1".to_string(), "hey2".to_string()], user0_token)
            .await
            .unwrap();

        let users = server.state.db.user_get_all().await.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "hey0");
    }
}
