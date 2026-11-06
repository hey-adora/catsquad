#[cfg(test)]
use catsquad_client::XMLSender;
use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
#[cfg(test)]
use catsquad_shared::Uuid;
use catsquad_shared::{Order, TimeRange, UserSearchFields, UserSearchRes};
use leptos::prelude::*;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct UsersState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub err_fetch: RwSignal<String>,
    pub err_remove: RwSignal<String>,
    pub users: RwSignal<Vec<UserSearchRes>>,
    pub param_search_order: RwSignal<Order>,
    pub param_search_limit: RwSignal<usize>,
    pub param_search_field_username: RwSignal<bool>,
    pub param_search_field_email: RwSignal<bool>,
    pub param_search_query: RwSignal<String>,

    pub check_all: RwSignal<bool>,
    pub checked_usernames: RwSignal<Vec<String>>,

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
            err_fetch: RwSignal::new(String::new()),
            err_remove: RwSignal::new(String::new()),
            users: RwSignal::new(Vec::new()),
            param_search_order: RwSignal::new(Order::ThreeTwoOne),
            param_search_limit: RwSignal::new(100),
            param_search_field_username: RwSignal::new(true),
            param_search_field_email: RwSignal::new(true),
            param_search_query: RwSignal::new(String::new()),
            check_all: RwSignal::new(false),
            checked_usernames: RwSignal::new(Vec::new()),
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

        self.err_fetch.update(|v| v.clear());
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
                self.err_fetch.set(err);
                return;
            }
        };
        self.users.set(users);
    }

    pub fn add_checked_username(&self, username: impl Into<String>) {
        let username = username.into();
        self.checked_usernames.update(|v| v.push(username));
    }

    pub fn remove_checked_username(&self, username: impl Into<String>) {
        let username = username.into();
        self.checked_usernames.update(|v| {
            let Some(pos) = v.iter().position(|v| **v == username) else {
                return;
            };
            v.swap_remove(pos);
        });
    }

    pub fn get_checked_usernames(&self) -> Vec<String> {
        if self.check_all.get_untracked() {
            self.users.with_untracked(|v| {
                v.iter()
                    .map(|v| v.username.clone())
                    .collect::<Vec<String>>()
            })
        } else {
            self.checked_usernames.get_untracked()
        }
    }

    /// extremely dangerious, rm -rf /
    pub async fn remove_users(&self) {
        let client = self.client.get_value();

        let usernames = self.get_checked_usernames();

        self.err_remove.update(|v| v.clear());

        let result = client
            .user_remove_by_usernames(usernames.clone())
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => (),
            Err(err) => {
                let err = format!("user_state error {err}");
                error!("{err}");
                self.err_remove.set(err);
                return;
            }
        };

        self.users.try_update(|v| {
            for username in usernames {
                if let Some(pos) = v.iter().position(|v| v.username == username) {
                    v.swap_remove(pos);
                }
            }
        });
    }
}

#[cfg(test)]
async fn test_user_state_remove_setup(
    db_name: &str,
) -> (
    Owner,
    catsquad_api::TestServer,
    UsersState<catsquad_client::AxumTestSender>,
    Uuid,
) {
    catsquad_log::init_log();
    let owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, db_name).await;

    let create_user = async |i: usize| {
        let (user, session) = server
            .user_add_full(
                format!("prime{i}"),
                format!("prime{i}@heyadora.com"),
                "235j4t49ngerigrog#IOTNOnfo",
            )
            .await;

        (user, session)
    };

    let (_user0, token0) = create_user(0).await;
    let (_user1, _token1) = create_user(10).await;
    let (_user2, _token2) = create_user(20).await;

    let user_state = UsersState::new(server.client.clone());

    (owner, server, user_state, token0)
}

#[cfg(test)]
#[tokio::test]
async fn test_user_state_remove_checked() {
    let (_owner, server, user_state, token0) =
        test_user_state_remove_setup("test_user_state_remove_checked").await;

    // TODO assert errors

    // assert success
    {
        use catsquad_api::{auth::create_auth_cookie_str, promote_admin};
        use catsquad_shared::uuid_to_str;
        use http::header;

        server.state.set_admin_username("prime0").await;
        promote_admin(server.state.clone()).await.unwrap();

        server
            .inject_header(header::COOKIE, create_auth_cookie_str(uuid_to_str(token0)))
            .await;

        user_state.fetch().await;

        let users_in_state = user_state.users.get_untracked();
        let users_in_db = server.state.db.user_get_all().await.unwrap();
        assert_eq!(users_in_state.len(), 3);
        assert_eq!(users_in_db.len(), 3);

        user_state.add_checked_username("prime10");
        user_state.add_checked_username("prime20");
        user_state.remove_users().await;

        let users_in_state = user_state.users.get_untracked();
        let users_in_db = server.state.db.user_get_all().await.unwrap();
        assert_eq!(users_in_state.len(), 1);
        assert_eq!(users_in_state[0].username, "prime0");
        assert_eq!(users_in_db.len(), 1);
        assert_eq!(users_in_db[0].username, "prime0");
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_state_remove_all() {
    let (_owner, server, user_state, token0) =
        test_user_state_remove_setup("test_user_state_remove_all").await;
    // TODO assert errors

    // assert success
    {
        use catsquad_api::{auth::create_auth_cookie_str, promote_admin};
        use catsquad_shared::uuid_to_str;
        use http::header;

        // setup, make prime0 admin
        {
            server.state.set_admin_username("prime0").await;
            promote_admin(server.state.clone()).await.unwrap();

            server
                .inject_header(header::COOKIE, create_auth_cookie_str(uuid_to_str(token0)))
                .await;
        }

        user_state.param_search_query.set("prime__ %".to_string());
        user_state.fetch().await;
        assert_eq!(user_state.users.with_untracked(|v| v.len()), 2);

        let users = server.state.db.user_get_all().await.unwrap();
        assert_eq!(users.len(), 3);

        user_state.check_all.set(true);
        user_state.remove_users().await;

        let users = server.state.db.user_get_all().await.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "prime0");
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_state_fetch() {
    use catsquad_api::auth::create_auth_cookie_str;
    use catsquad_api::promote_admin;
    use catsquad_shared::{PostState, uuid_to_str};
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_user_state_fetch").await;

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

        assert_eq!(user_state.err_fetch.get_untracked(), "");
        assert_eq!(user_state.users.get_untracked().len(), 0);
        user_state
    };

    // assert error: no acc
    {
        user_state.fetch().await;

        assert!(!user_state.err_fetch.get_untracked().is_empty());
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

        assert!(!user_state.err_fetch.get_untracked().is_empty());
        assert_eq!(user_state.users.get_untracked().len(), 0);
    }

    // assert success
    {
        server.state.set_admin_username("prime1").await;
        promote_admin(server.state.clone()).await.unwrap();

        user_state.fetch().await;

        assert_eq!(user_state.err_fetch.get_untracked(), "");
        assert_eq!(user_state.users.get_untracked().len(), 1);
        assert_eq!(user_state.users.get_untracked()[0].username, "prime1");
    }
}
