use crate::state::AppState;
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbUserGet, DbUserSearch, DbUserSearchErr};
use catsquad_shared::{
    Order, ROLE_VALUE_PERMISSION_USER_SEARCH, TimeRange, UserSearchErr, UserSearchParams,
    UserSearchRes, role_extract_permission_user_search,
};

pub fn from_db_user_search(value: Vec<DbUserSearch>) -> Vec<UserSearchRes> {
    value
        .into_iter()
        .map(|value| UserSearchRes {
            username: value.username,
            pfp_hash: value.pfp_image_hash,
            pfp_proccesed: value.pfp_image_proccesed,
            followers: value.followers_count,
            following: value.following_count,
            likes: value.likes_count,
            support: value.support,
            aboutme: value.aboutme,
            created_at: value.created_at,
        })
        .collect()
}

fn from_db_user_search_err(value: DbUserSearchErr) -> UserSearchErr {
    match value {
        DbUserSearchErr::Db(_) => UserSearchErr::InternalServer,
    }
}

fn status_code(result: &Result<Vec<UserSearchRes>, UserSearchErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserSearchErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserSearchErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn user_search(
    db_user: Extension<DbUserGet>,
    State(app): State<AppState>,
    Path(params): Path<UserSearchParams>,
) -> impl IntoResponse {
    let inner = async || -> Result<Vec<UserSearchRes>, UserSearchErr> {
        let permission_role_add = role_extract_permission_user_search(&db_user.roles_values);
        if !permission_role_add {
            return Err(UserSearchErr::Unauthorized(format!(
                "user {} doesnt have permission {}",
                db_user.username, ROLE_VALUE_PERMISSION_USER_SEARCH
            )));
        }

        let time_range_value = params.time.unwrap_or(0);
        let time_range = params.range.unwrap_or(TimeRange::MoreOrEqual);
        let order = params.order.unwrap_or(Order::ThreeTwoOne);
        let limit = params.limit.unwrap_or(100);

        let users = app
            .db
            .user_search(time_range_value, limit, time_range, order)
            .await
            .map_err(from_db_user_search_err)?;

        Ok(from_db_user_search(users))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(any(test, feature = "test_server"))]
mod test_utils {
    use crate::{TestServer, auth::create_auth_cookie_str};
    use axum::http::header;
    use catsquad_shared::{self as cs, Order, TimeRange, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn user_search(
            &self,
            time: u64,
            limit: usize,
            range: TimeRange,
            order: Order,
            session_token: Uuid,
        ) -> Result<Vec<cs::UserSearchRes>, cs::UserSearchErr> {
            self.client
                .user_search(time, limit, range, order)
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
async fn test_api_user_search() {
    use catsquad_log::prelude::*;
    init_log();
    let server = crate::TestServer::new(0, "test_api_user_search").await;

    server.state.set_admin_username("hey0").await;

    let (user_one, session_key) = server
        .user_add_full("hey0", "hey0@heyadora.com", "PAss$ord11111")
        .await;

    // server.state.db.role_update_value(0, role_id, values);

    for i in 1..10 {
        let username = format!("hey{i}");
        let email = format!("hey{i}@heyadora.com");
        let (_user_add, _session_key) =
            server.user_add_full(username, email, "PAss$ord11111").await;
    }

    // assert error
    {
        let result = server
            .user_search(
                0,
                10,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                0_u128.to_be_bytes(),
            )
            .await;

        assert!(matches!(result, Err(UserSearchErr::Unauthorized(_))));
    }

    // assert success
    {
        let users = server
            .user_search(
                0,
                10,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                session_key,
            )
            .await
            .unwrap();

        assert_eq!(users.len(), 10);
    }
    // let result = server.user_get_by_username("hey").await.unwrap();
    // assert_eq!(user_add.username, result.username);

    // let result = server.user_get_by_username("hey1").await;
    // assert_eq!(result, Err(UserGetByUsernameErr::NotFound));
}
