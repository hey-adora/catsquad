use crate::{api::role_add::from_db_role, auth::SessionTokenV2, state::AppState};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::DbRoleGetAllErr;
use catsquad_shared::{RoleGetAllErr, RoleRes, RoleValue};

fn from_db_role_get_all(value: DbRoleGetAllErr) -> RoleGetAllErr {
    match value {
        DbRoleGetAllErr::Db(_) => RoleGetAllErr::InternalServer,
    }
}

fn status_code(result: &Result<Vec<RoleRes>, RoleGetAllErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(RoleGetAllErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn role_get_all(State(app): State<AppState>) -> impl IntoResponse {
    let inner = async || -> Result<Vec<RoleRes>, RoleGetAllErr> {
        let roles = app
            .db
            .role_get_all()
            .await
            .map_err(from_db_role_get_all)?
            .into_iter()
            .map(from_db_role)
            .collect::<Vec<RoleRes>>();

        Ok(roles)
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::TestServer;
    use catsquad_shared::{self as cs, RoleRes};

    impl TestServer {
        pub async fn role_get_all(&self) -> Result<Vec<RoleRes>, cs::RoleGetAllErr> {
            self.client.role_get_all().send().await.into_json().await
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_api_role_get_all() {
    use catsquad_log::prelude::*;
    use catsquad_shared::RoleValue;
    use catsquad_shared::SessionTokenKind;

    init_log();
    let server = crate::TestServer::new(0, "test_api_role_get_all").await;

    // assert success
    {
        server
            .state
            .db
            .role_add(
                0,
                SessionTokenKind::Root,
                "mod",
                vec![RoleValue::PermissionRoleAdd],
            )
            .await
            .unwrap();

        let result = server.role_get_all().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].values.len(), 1);
        assert_eq!(result[0].values[0], RoleValue::PermissionRoleAdd.as_str());
    }
}
