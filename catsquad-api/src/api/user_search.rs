use crate::{auth::SessionTokenV2, state::AppState};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use catsquad_db::{DbUserSearch, DbUserSearchBuilderErr};
use catsquad_log::prelude::*;
use catsquad_shared::{
    Order, TimeRange, UserSearchErr, UserSearchFields, UserSearchParams, UserSearchRes,
};

pub fn from_db_user_search(value: Vec<DbUserSearch>) -> Vec<UserSearchRes> {
    value
        .into_iter()
        .map(|value| UserSearchRes {
            username: value.username,
            state: value.state,
            email: value.email,
            pfp_hash: value.pfp_image_hash,
            pfp_proccesed: value.pfp_image_proccesed,
            followers: value.followers_count,
            following: value.following_count,
            likes: value.likes_count,
            support: value.support,
            aboutme: value.aboutme,
            ip: value.ip.to_string(),
            agent: value.agent,
            created_at: value.created_at,
        })
        .collect()
}

fn from_db_user_search_err(value: DbUserSearchBuilderErr) -> UserSearchErr {
    match value {
        DbUserSearchBuilderErr::Unauthorized(err) => UserSearchErr::Unauthorized(err),
        DbUserSearchBuilderErr::InternalServerErr => UserSearchErr::InternalServer,
        DbUserSearchBuilderErr::Db(_) => UserSearchErr::InternalServer,
    }
}

fn status_code(result: &Result<Vec<UserSearchRes>, UserSearchErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(UserSearchErr::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
        Err(UserSearchErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn prepare_user_search_params(
    params: UserSearchParams,
) -> (
    u64,
    usize,
    usize,
    TimeRange,
    Order,
    UserSearchFields,
    String,
) {
    let time_range_value = params.time.unwrap_or(0);
    let time_range = params.range.unwrap_or(TimeRange::MoreOrEqual);
    let order = params.order.unwrap_or(Order::ThreeTwoOne);
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or(0);

    let mut search_fields = UserSearchFields::default();
    if let Some(search_field_username) = params.search_field_username {
        search_fields.username = search_field_username;
    };
    if let Some(search_field_email) = params.search_field_email {
        search_fields.email = search_field_email;
    };
    let search_query = params.search_query.unwrap_or_default();
    let search_query = search_query.trim();

    (
        time_range_value,
        limit,
        offset,
        time_range,
        order,
        search_fields,
        search_query.to_string(), // TODO optimize this ugly thing
    )
}

pub async fn user_search(
    auth_session_token: SessionTokenV2,
    State(app): State<AppState>,
    Query(params): Query<UserSearchParams>,
) -> impl IntoResponse {
    trace!("user_search received params {params:#?}");
    let inner = async || -> Result<Vec<UserSearchRes>, UserSearchErr> {
        let params = prepare_user_search_params(params);

        let users = app
            .db
            .user_search_users(
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
    use catsquad_shared::{self as cs, Order, TimeRange, UserSearchFields, Uuid, uuid_to_str};

    impl TestServer {
        pub async fn user_search(
            &self,
            time: u64,
            limit: usize,
            offset: usize,
            range: TimeRange,
            order: Order,
            search_fields: UserSearchFields,
            search_query: impl Into<String>,
            session_token: Uuid,
        ) -> Result<Vec<cs::UserSearchRes>, cs::UserSearchErr> {
            self.client
                .user_search(
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
async fn test_api_user_search() {
    use catsquad_log::prelude::*;
    use catsquad_shared::{UUID_ZERO, UserSearchFields, Uuid};

    init_log();
    let server = crate::TestServer::new(0, "test_api_user_search").await;

    let (_user_one, session_key) = server
        .user_add_full("hey0", "hey0@heyadora.com", "PAss$ord11111")
        .await;

    for i in 1..10 {
        let username = format!("hey{i}");
        let email = format!("hey{i}@heyadora.com");
        let (_user_add, _session_key) =
            server.user_add_full(username, email, "PAss$ord11111").await;
    }

    // assert unauthorized
    {
        let search = async |token: Uuid| {
            server
                .user_search(
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

        let result = search(UUID_ZERO).await;
        assert!(matches!(result, Err(UserSearchErr::Unauthorized(_))));

        let result = search(session_key).await;
        assert!(matches!(result, Err(UserSearchErr::Unauthorized(_))));

        let result = server
            .client
            .user_search(
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
        assert!(matches!(result, Err(UserSearchErr::Unauthorized(_))));
    }

    server.admin("hey0").await;

    // assert success
    {
        let users = server
            .user_search(
                0,
                10,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
                session_key,
            )
            .await
            .unwrap();

        assert_eq!(users.len(), 10);

        let users = server
            .user_search(
                0,
                5,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "",
                session_key,
            )
            .await
            .unwrap();

        assert_eq!(users.len(), 5);
    }

    // assert success search feature
    {
        let users = server
            .user_search(
                0,
                5,
                0,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
                UserSearchFields::default(),
                "hey0%",
                session_key,
            )
            .await
            .unwrap();

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "hey0");
    }
}
