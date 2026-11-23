use crate::{api::user_search::prepare_user_search_params, auth::SessionTokenV2, state::AppState};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::DbUserSearchBuilderErr;
use catsquad_log::prelude::*;
use catsquad_shared::{UserPaginationErr, UserPaginationRes, UserSearchParams};

fn from_db_user_pagination_err(value: DbUserSearchBuilderErr) -> UserPaginationErr {
    match value {
        DbUserSearchBuilderErr::Unauthorized(err) => UserPaginationErr::Unauthorized(err),
        DbUserSearchBuilderErr::InternalServerErr => UserPaginationErr::InternalServer,
        DbUserSearchBuilderErr::Db(_) => UserPaginationErr::InternalServer,
    }
}

fn status_code(result: &Result<UserPaginationRes, UserPaginationErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserPaginationErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserPaginationErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_pagination(
    auth_session_token: SessionTokenV2,
    State(app): State<AppState>,
    Query(params): Query<UserSearchParams>,
) -> impl IntoResponse {
    trace!("user_search received params {params:#?}");
    let inner = async || -> Result<UserPaginationRes, UserPaginationErr> {
        let params = prepare_user_search_params(params);

        let total_pages = app
            .db
            .user_search_pagination(
                auth_session_token.into(),
                params.0,
                params.1,
                params.2,
                params.3,
                params.4,
                params.5,
                params.6,
            )
            .await
            .map_err(from_db_user_pagination_err)?;

        Ok(UserPaginationRes {
            total_count: total_pages as usize,
        })
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(any(test, feature = "test_server"))]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Order, TimeRange, UserSearchFields, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn user_pagination(
            &self,
            time: u64,
            limit: usize,
            offset: usize,
            range: TimeRange,
            order: Order,
            search_fields: UserSearchFields,
            search_query: impl Into<String>,
            session_token: Uuid,
        ) -> Result<cs::UserPaginationRes, cs::UserPaginationErr> {
            self.client
                .user_pagination(
                    time,
                    limit,
                    offset,
                    range,
                    order,
                    search_fields,
                    search_query,
                )
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
async fn test_api_user_pagination() {
    use catsquad_log::prelude::*;
    use catsquad_shared::{Order, TimeRange, UUID_ZERO, UserPaginationErr, UserSearchFields, Uuid};

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_pagination").await;

    let (_user_one, session_key) = server
        .user_add_full("hey0", "hey0@heyadora.com", "PAss$ord11111")
        .await;

    for i in 1..10 {
        let username = format!("hey{i}");
        let email = format!("hey{i}@heyadora.com");
        let (_user_add, _session_key) =
            server.user_add_full(username, email, "PAss$ord11111").await;
    }

    let pagination = async |token: Uuid| {
        server
            .user_pagination(
                0,
                10,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
                token,
            )
            .await
    };

    // assert unauthorized
    {
        let result = pagination(UUID_ZERO).await;
        assert!(matches!(result, Err(UserPaginationErr::Unauthorized(_))));

        let result = pagination(session_key).await;
        assert!(matches!(result, Err(UserPaginationErr::Unauthorized(_))));

        let result = server
            .client
            .user_pagination(
                0,
                10,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
            )
            .send()
            .await
            .into_json()
            .await;
        assert!(matches!(result, Err(UserPaginationErr::Unauthorized(_))));
    }

    server.admin("hey0").await;

    // assert success
    {
        let result = pagination(session_key).await.unwrap();
        assert_eq!(result.total_count, 10);
    }
}
