use crate::state::AppState;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbRoleRemoveErr, DbUserGet};
use catsquad_shared::{
    ROLE_VALUE_PERMISSION_ROLE_REMOVE, RoleRemoveErr, RoleRemoveParams,
    role_extract_permission_role_remove,
};

pub fn from_db_role_add_err(value: DbRoleRemoveErr) -> RoleRemoveErr {
    match value {
        DbRoleRemoveErr::NotFound => RoleRemoveErr::NotFound,
        DbRoleRemoveErr::Db(_) => RoleRemoveErr::InternalServer,
    }
}

fn status_code(result: &Result<(), RoleRemoveErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(RoleRemoveErr::NotFound) => StatusCode::BAD_REQUEST,
        Err(RoleRemoveErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(RoleRemoveErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(RoleRemoveErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn role_remove(
    db_user: Extension<DbUserGet>,
    State(app): State<AppState>,
    Path(params): Path<RoleRemoveParams>,
) -> impl IntoResponse {
    let inner = async || -> Result<(), RoleRemoveErr> {
        let permission_role_add = role_extract_permission_role_remove(&db_user.roles_values);

        if !permission_role_add {
            return Err(RoleRemoveErr::Unauthorized(format!(
                "user {} doesnt have permission {}",
                db_user.username, ROLE_VALUE_PERMISSION_ROLE_REMOVE
            )));
        }

        let role_id = params.role_name;

        app.db
            .role_remove(role_id)
            .await
            .map_err(from_db_role_add_err)?;

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
    use catsquad_shared::{self as cs, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn role_remove(
            &self,
            role_id: i64,
            session_token: Uuid,
        ) -> Result<(), cs::RoleRemoveErr> {
            self.client
                .role_remove(role_id)
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
async fn test_api_role_remove() {
    use catsquad_log::prelude::*;
    use catsquad_shared::RoleValue;

    init_log();
    let server = crate::TestServer::new(0, "test_api_role_remove").await;

    let email = "hey@heyadora.com";
    let password = "1nnerogGeron@@$";
    let (_user, session_key) = server.user_add_full("hey", email, password).await;

    // setup, create admin role
    let role_admin = {
        let role_admin = server
            .state
            .db
            .role_add(0, "admin", vec![RoleValue::PermissionRoleAdd])
            .await
            .unwrap();

        let _role_admin_2 = server
            .state
            .db
            .role_add(0, "admin2", vec![RoleValue::PermissionRoleAdd])
            .await
            .unwrap();

        server
            .state
            .db
            .user_update_role_add(0, "hey", "admin")
            .await
            .unwrap();

        role_admin
    };

    // assert errors
    {
        let result = server.role_remove(0, session_key).await;
        assert!(matches!(result, Err(RoleRemoveErr::Unauthorized(_))));

        // update admin role, grant role_remove permissinos
        server
            .state
            .db
            .role_update_value(
                0,
                role_admin.id,
                vec![
                    RoleValue::PermissionRoleAdd,
                    RoleValue::PermissionRoleRemove,
                ],
            )
            .await
            .unwrap();

        let result = server.role_remove(0, session_key).await;
        assert!(matches!(result, Err(RoleRemoveErr::NotFound)));
    }

    // assert success
    {
        server.state.set_time(1);

        server
            .role_remove(role_admin.id, session_key)
            .await
            .unwrap();

        let roles = server.state.db.role_get_all().await.unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "admin2");
    }
}
