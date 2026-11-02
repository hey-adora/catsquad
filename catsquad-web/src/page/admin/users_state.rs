use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{Order, TimeRange, UserSearchFields, UserSearchRes};
use leptos::prelude::*;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct UsersState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub err_general: RwSignal<String>,
    pub users: RwSignal<Vec<UserSearchRes>>,
    pub param_search_order: RwSignal<Order>,
    pub param_search_limit: RwSignal<usize>,
    pub param_search_field_username: RwSignal<bool>,
    pub param_search_field_email: RwSignal<bool>,
    pub param_search_query: RwSignal<String>,
    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Copy for UsersState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

impl<TSender> UsersState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            users: RwSignal::new(Vec::new()),
            param_search_order: RwSignal::new(Order::ThreeTwoOne),
            param_search_limit: RwSignal::new(100),
            param_search_field_username: RwSignal::new(true),
            param_search_field_email: RwSignal::new(true),
            param_search_query: RwSignal::new(String::new()),
            client: StoredValue::new_local(client),
        }
    }

    pub async fn fetch(&self) {
        let client = self.client.get_value();
        let param_search_order = self.param_search_order.get_untracked();
        let param_search_limit = self.param_search_limit.get_untracked();
        let param_search_field_username = self.param_search_field_username.get_untracked();
        let param_search_field_email = self.param_search_field_email.get_untracked();
        let param_search_field = UserSearchFields {
            username: param_search_field_username,
            email: param_search_field_email,
        };
        let param_search_query = self.param_search_query.get_untracked();

        self.err_general.update(|v| v.clear());
        let result = client
            .user_search(
                0,
                param_search_limit,
                0,
                TimeRange::MoreOrEqual,
                param_search_order,
                param_search_field,
                param_search_query,
            )
            .send()
            .await
            .into_json()
            .await;
        let users = match result {
            Ok(v) => v,
            Err(err) => {
                let err = format!("user_state error {err}");
                error!("{err}");
                self.err_general.set(err);
                return;
            }
        };
        self.users.set(users);
    }
}
#[cfg(test)]
#[tokio::test]
async fn test_user_state() {
    use catsquad_api::auth::create_auth_cookie_str;
    use catsquad_api::promote_admin;
    use catsquad_shared::{PostState, uuid_to_str};
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_user_state").await;

    let (_user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    // assert init
    let user_state = {
        let user_state = UsersState::new(server.client.clone());

        assert_eq!(user_state.err_general.get_untracked(), "");
        assert_eq!(user_state.users.get_untracked().len(), 0);
        user_state
    };

    // assert error: no acc
    {
        user_state.fetch().await;

        assert!(!user_state.err_general.get_untracked().is_empty());
        assert_eq!(user_state.users.get_untracked().len(), 0);
    }

    // assert error: no permission
    {
        server
            .inject_header(
                header::COOKIE,
                create_auth_cookie_str(uuid_to_str(session1)),
            )
            .await;

        user_state.fetch().await;

        assert!(!user_state.err_general.get_untracked().is_empty());
        assert_eq!(user_state.users.get_untracked().len(), 0);
    }

    // assert success
    {
        server.state.set_admin_username("prime1").await;
        promote_admin(server.state.clone()).await.unwrap();

        user_state.fetch().await;

        assert_eq!(user_state.err_general.get_untracked(), "");
        assert_eq!(user_state.users.get_untracked().len(), 1);
        assert_eq!(user_state.users.get_untracked()[0].username, "prime1");
    }
}
