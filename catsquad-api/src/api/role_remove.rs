use crate::state::AppState;
use axum::{
    Extension, Form, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

pub fn from_db_role_add_err(value: DbRoleRemoveErr) -> RoleAddErr {
    match value {
        DbRoleAddErr::Db(_) => RoleAddErr::InternalServer,
    }
}

fn status_code(result: &Result<(), RoleAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(RoleAddErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(RoleAddErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(RoleAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn role_remove(
    db_user: Extension<DbUserGet>,
    State(app): State<AppState>,
    Path(params): Path<RoleAddPrams>,
    Form(req): Form<RoleAddReq>,
) -> impl IntoResponse {
    let inner = async || -> Result<RoleRes, RoleAddErr> {
        let permission_role_add = role_extract_permission_role_add(&db_user.roles_values);

        if !permission_role_add {
            return Err(RoleAddErr::Unauthorized(format!(
                "user {} doesnt have permission {}",
                db_user.username, ROLE_VALUE_PERMISSION_ROLE_ADD
            )));
        }

        let time = app.get_time_micro();
        let role_name = params.role_name;
        let role_values = req.role_values;

        let post_like = app
            .db
            .role_add(time, role_name, role_values)
            .await
            .map_err(from_db_role_add_err)?;

        Ok(from_db_role(post_like))
    };

    let result = inner().await;
    let status_code = status_code(&result);
    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use std::fmt::Display;

    use axum::http::header;
    use catsquad_shared::{self as cs, RoleValue, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn role_add(
            &self,
            role_name: impl Display,
            role_values: Vec<RoleValue>,
            session_token: Uuid,
        ) -> Result<cs::RoleRes, cs::RoleAddErr> {
            self.client
                .role_add(role_name, role_values)
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
async fn test_api_role_add() {
    use catsquad_log::prelude::*;
    use catsquad_shared::RoleValue;

    init_log();
    let server = crate::TestServer::new(0, "test_api_post_like_add").await;

    let email = "hey@heyadora.com";
    let password = "1nnerogGeron@@$";
    let (_user, session_key) = server.user_add_full("hey", email, password).await;

    // assert errors
    {
        let result = server
            .role_add("mod", vec![RoleValue::PermissionRoleRemove], session_key)
            .await;

        assert!(matches!(result, Err(RoleAddErr::Unauthorized(_))));
    }

    // assert success
    {
        server
            .state
            .db
            .role_add(0, "admin", vec![RoleValue::PermissionRoleAdd])
            .await
            .unwrap();

        server
            .state
            .db
            .user_update_role_add(0, "hey", "admin")
            .await
            .unwrap();

        server.state.set_time(1);

        let _role_one = server
            .role_add("mod", vec![RoleValue::PermissionRoleRemove], session_key)
            .await
            .unwrap();

        let roles = server.state.db.role_get_all().await.unwrap();
        assert_eq!(roles.len(), 2);
        assert_eq!(roles[0].values.len(), 1);
        assert_eq!(roles[1].values.len(), 1);
        assert_eq!(roles[0].name, "mod");
        assert_eq!(roles[1].name, "admin");
    }
}
