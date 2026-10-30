use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{Order, TimeRange, UserSearchRes};
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
            client: StoredValue::new_local(client),
        }
    }

    pub async fn fetch(&self) {
        let client = self.client.get_value();
        self.err_general.update(|v| v.clear());
        let result = client
            .user_search(0, 1000, 0, TimeRange::MoreOrEqual, Order::ThreeTwoOne)
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
