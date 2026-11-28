use crate::{auth::SessionTokenV2, state::AppState};
use axum::{
    Form, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::DbRoleUpdateValueErr;
use catsquad_shared::{
    RoleUpdateValueErr, RoleUpdateValueParams, RoleUpdateValueReq, SessionTokenKind,
};

pub fn from_db_role_add_err(value: DbRoleUpdateValueErr) -> RoleUpdateValueErr {
    match value {
        DbRoleUpdateValueErr::DuplicateValue => RoleUpdateValueErr::DuplicateValue,
        DbRoleUpdateValueErr::NotFound => RoleUpdateValueErr::NotFound,
        DbRoleUpdateValueErr::Unauthorized(err) => RoleUpdateValueErr::Unauthorized(err),
        DbRoleUpdateValueErr::Db(_) => RoleUpdateValueErr::InternalServer,
    }
}

fn status_code(result: &Result<(), RoleUpdateValueErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(RoleUpdateValueErr::NotFound) => StatusCode::NOT_FOUND,
        Err(RoleUpdateValueErr::DuplicateValue) => StatusCode::BAD_REQUEST,
        Err(RoleUpdateValueErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(RoleUpdateValueErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(RoleUpdateValueErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn role_update_value(
    auth_session_token: SessionTokenV2,
    State(app): State<AppState>,
    Path(params): Path<RoleUpdateValueParams>,
    Form(req): Form<RoleUpdateValueReq>,
) -> impl IntoResponse {
    let time = app.get_time_micro();

    let inner = async || -> Result<(), RoleUpdateValueErr> {
        let role_id = params.role_name;
        let role_values = req.role_values;

        app.db
            .role_update_value(auth_session_token.into(), time, role_id, role_values)
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
    use std::fmt::Display;

    use axum::http::header;
    use catsquad_shared::{self as cs, RoleValue, Uuid, uuid_to_str};

    use crate::{TestServer, auth::create_auth_cookie_str};

    impl TestServer {
        pub async fn role_update_value(
            &self,
            role_id: i64,
            role_values: Vec<RoleValue>,
            session_token: Uuid,
        ) -> Result<cs::RoleRes, cs::RoleAddErr> {
            self.client
                .role_update_value(role_name, role_values)
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
async fn test_api_role_update_value() {
    use catsquad_log::prelude::*;
    use catsquad_shared::RoleValue;

    init_log();
    let server = crate::TestServer::new(0, "test_api_role_add").await;

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
        use catsquad_shared::SessionTokenKind;

        server
            .state
            .db
            .role_add(
                0,
                SessionTokenKind::Root,
                "admin",
                vec![RoleValue::PermissionRoleAdd],
            )
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
