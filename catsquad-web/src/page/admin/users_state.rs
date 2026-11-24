#[cfg(test)]
use catsquad_client::XMLSender;
use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
#[cfg(test)]
use catsquad_shared::Uuid;
use catsquad_shared::{Order, TimeRange, UserSearchFields, UserSearchRes, UserState};
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
    pub err_state: RwSignal<String>,
    pub users: RwSignal<Vec<UserSearchRes>>,
    pub param_search_order: RwSignal<Order>,
    pub param_search_limit: RwSignal<usize>,
    pub param_search_field_username: RwSignal<bool>,
    pub param_search_field_email: RwSignal<bool>,
    pub param_search_query: RwSignal<String>,
    pub param_search_offset: RwSignal<usize>,

    pub page_total_rows: RwSignal<usize>,

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

// fn current_page(limit: usize, offset: usize) -> usize {
//     offset / limit
// }

// fn total_pages(limit: usize, total: usize) -> usize {
//     total / limit
// }

// fn test_page_math(

// )

impl<TSender> UsersState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            err_fetch: RwSignal::new(String::new()),
            err_remove: RwSignal::new(String::new()),
            err_state: RwSignal::new(String::new()),
            users: RwSignal::new(Vec::new()),
            param_search_order: RwSignal::new(Order::ThreeTwoOne),
            param_search_limit: RwSignal::new(100),
            param_search_field_username: RwSignal::new(true),
            param_search_field_email: RwSignal::new(true),
            param_search_query: RwSignal::new(String::new()),
            check_all: RwSignal::new(false),
            checked_usernames: RwSignal::new(Vec::new()),
            param_search_offset: RwSignal::new(0),
            page_total_rows: RwSignal::new(0),
            client: StoredValue::new_local(client),
        }
    }

    pub fn current_page(&self) -> usize {
        self.param_search_offset.get() / self.param_search_limit.get()
    }

    pub fn max_page(&self) -> usize {
        self.page_total_rows.get() / self.param_search_limit.get()
    }

    pub fn next_page(&self) {
        let limit = self.param_search_limit.get_untracked();
        self.param_search_offset.update(|offset| *offset += limit);
    }

    pub fn prev_page(&self) {
        let limit = self.param_search_limit.get_untracked();
        self.param_search_offset
            .update(|offset| *offset = offset.saturating_sub(limit));
    }

    pub fn is_first_page(&self) -> bool {
        self.param_search_offset.get() == 0
    }

    pub fn is_last_page(&self) -> bool {
        (self.param_search_limit.get() + self.param_search_offset.get())
            >= self.page_total_rows.get()
    }

    pub async fn init(&self) {
        self.get_pagination().await;
        self.get_rows().await;
    }

    pub async fn get_rows(&self) {
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
        let param_search_offset = self.param_search_offset.get_untracked();

        self.err_fetch.update(|v| v.clear());

        // get users
        {
            let result = client
                .user_search(
                    0,
                    param_search_limit,
                    param_search_offset,
                    TimeRange::MoreOrEqual,
                    param_search_order.clone(),
                    param_search_field.clone(),
                    param_search_query.clone(),
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
    }

    pub async fn get_pagination(&self) {
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

        // get pagination
        {
            let result = client
                .user_pagination(
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
            let pagination = match result {
                Ok(v) => v,
                Err(err) => {
                    let err = format!("user_state error {err}");
                    error!("{err}");
                    self.err_fetch.set(err);
                    return;
                }
            };
            self.page_total_rows.set(pagination.total_count);
        }
    }

    pub fn clear_checked(&self) {
        self.checked_usernames.update(|v| v.clear());
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

    pub async fn suspend_users(&self) {
        self.update_users_state(UserState::Suspended).await;
    }
    pub async fn active_users(&self) {
        self.update_users_state(UserState::Active).await;
    }
    pub async fn update_users_state(&self, new_state: UserState) {
        let client = self.client.get_value();

        let usernames = self.get_checked_usernames();

        self.err_state.update(|v| v.clear());

        let result = client
            .user_update_state(usernames.clone(), new_state)
            .send()
            .await
            .into_json()
            .await;

        let users = match result {
            Ok(v) => v,
            Err(err) => {
                let err = format!("user_state error {err}");
                error!("{err}");
                self.err_remove.set(err);
                return;
            }
        };

        // TODO compare returned users, see if all got updated, report that were not found

        self.users.try_update(|v| {
            for user in users {
                let username = user.username;
                if let Some(pos) = v.iter().position(|v| v.username == username) {
                    let Some(user) = v.get_mut(pos) else {
                        break;
                    };
                    user.state = new_state.to_string();
                }
            }
        });
    }
}

#[cfg(test)]
async fn test_user_state_setup(
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
        server.state.set_time(i as u64);
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
    server.state.set_time(4);

    let user_state = UsersState::new(server.client.clone());

    (owner, server, user_state, token0)
}

#[cfg(test)]
#[tokio::test]
async fn test_user_state_update_checked() {
    let (_owner, server, user_state, token0) =
        test_user_state_setup("test_user_state_update_checked").await;

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

        user_state.get_rows().await;

        let users_in_state = user_state.users.get_untracked();
        let users_in_db = server.state.db.user_get_all().await.unwrap();
        assert_eq!(users_in_state.len(), 3);
        assert_eq!(users_in_db.len(), 3);

        assert_eq!(users_in_state[0].state, UserState::Active.as_str());
        assert_eq!(users_in_state[1].state, UserState::Active.as_str());
        assert_eq!(users_in_state[2].state, UserState::Active.as_str());

        assert_eq!(users_in_state[0].username, "prime20");
        assert_eq!(users_in_state[1].username, "prime10");
        assert_eq!(users_in_state[2].username, "prime0");

        assert_eq!(users_in_db[0].state, UserState::Active.as_str());
        assert_eq!(users_in_db[1].state, UserState::Active.as_str());
        assert_eq!(users_in_db[2].state, UserState::Active.as_str());

        assert_eq!(users_in_db[0].username, "prime20");
        assert_eq!(users_in_db[1].username, "prime10");
        assert_eq!(users_in_db[2].username, "prime0");

        user_state.add_checked_username("prime10");
        user_state.add_checked_username("prime20");

        user_state.suspend_users().await;

        let users_in_state = user_state.users.get_untracked();
        let users_in_db = server.state.db.user_get_all().await.unwrap();
        assert_eq!(users_in_state.len(), 3);
        assert_eq!(users_in_db.len(), 3);

        assert_eq!(users_in_state[0].state, UserState::Suspended.as_str());
        assert_eq!(users_in_state[1].state, UserState::Suspended.as_str());
        assert_eq!(users_in_state[2].state, UserState::Active.as_str());

        assert_eq!(users_in_state[0].username, "prime20");
        assert_eq!(users_in_state[1].username, "prime10");
        assert_eq!(users_in_state[2].username, "prime0");

        assert_eq!(users_in_db[0].state, UserState::Suspended.as_str());
        assert_eq!(users_in_db[1].state, UserState::Suspended.as_str());
        assert_eq!(users_in_db[2].state, UserState::Active.as_str());

        assert_eq!(users_in_db[0].username, "prime20");
        assert_eq!(users_in_db[1].username, "prime10");
        assert_eq!(users_in_db[2].username, "prime0");
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_state_remove_checked() {
    let (_owner, server, user_state, token0) =
        test_user_state_setup("test_user_state_remove_checked").await;

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

        user_state.get_rows().await;

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
        test_user_state_setup("test_user_state_remove_all").await;
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
        user_state.get_rows().await;
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
    use catsquad_shared::uuid_to_str;
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

    server.state.set_time(1);

    let (_user2, session2) = server
        .user_add_full(
            "prime2",
            "prime2@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    server.state.set_time(2);

    let (_user3, session3) = server
        .user_add_full(
            "prime3",
            "prime3@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    // assert init
    let user_state = {
        let user_state = UsersState::new(server.client.clone());
        user_state.param_search_limit.set(2);

        assert_eq!(user_state.err_fetch.get_untracked(), "");
        assert_eq!(user_state.users.get_untracked().len(), 0);
        user_state
    };

    // assert error: no acc
    {
        user_state.get_rows().await;

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

        user_state.get_rows().await;

        assert!(!user_state.err_fetch.get_untracked().is_empty());
        assert_eq!(user_state.users.get_untracked().len(), 0);
        assert_eq!(user_state.page_total_rows.get_untracked(), 0);
    }

    // assert success
    {
        server.state.set_admin_username("prime1").await;
        promote_admin(server.state.clone()).await.unwrap();

        user_state.init().await;

        assert_eq!(user_state.err_fetch.get_untracked(), "");

        assert_eq!(user_state.param_search_offset.get_untracked(), 0);
        assert_eq!(user_state.page_total_rows.get_untracked(), 3);

        assert_eq!(user_state.current_page(), 0);
        assert_eq!(user_state.max_page(), 1);
        assert!(user_state.is_first_page());
        assert!(!user_state.is_last_page());

        assert_eq!(user_state.users.get_untracked().len(), 2);
        assert_eq!(user_state.users.get_untracked()[0].username, "prime3");
        assert_eq!(user_state.users.get_untracked()[1].username, "prime2");

        user_state.next_page();
        user_state.get_rows().await;

        assert_eq!(user_state.param_search_offset.get_untracked(), 2);
        assert_eq!(user_state.page_total_rows.get_untracked(), 3);

        assert_eq!(user_state.current_page(), 1);
        assert_eq!(user_state.max_page(), 1);
        assert!(!user_state.is_first_page());
        assert!(user_state.is_last_page());

        assert_eq!(user_state.users.get_untracked().len(), 1);
        assert_eq!(user_state.users.get_untracked()[0].username, "prime1");
    }
}
