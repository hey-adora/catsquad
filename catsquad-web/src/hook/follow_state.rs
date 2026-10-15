use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use leptos::prelude::*;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub struct FollowState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub err_general: RwSignal<String>,
    pub followed: RwSignal<bool>,
    pub following_user_username: Signal<String>,
    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Copy for FollowState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

impl<TSender> FollowState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new_with_state(
        client: Client<TSender>,
        following_user_username: Signal<String>,
        followed: RwSignal<bool>,
    ) -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            followed,
            following_user_username,
            client: StoredValue::new_local(client),
        }
    }

    pub async fn follow_toggle(&self) {
        let client = self.client.get_value();
        let followed = self.followed.get_untracked();
        let following_user_username = self.following_user_username.get_untracked();

        if following_user_username.is_empty() {
            return;
        }

        self.err_general.try_update(|v| v.clear());

        match followed {
            true => {
                let result = client
                    .follow_remove(following_user_username)
                    .send()
                    .await
                    .into_json()
                    .await;
                match result {
                    Ok(_result) => {
                        self.followed.try_set(false);
                    }
                    Err(err) => {
                        error!("follow_state: {err}");
                        self.err_general.try_set(err.to_string());
                    }
                };
            }
            false => {
                let result = client
                    .follow_add(following_user_username)
                    .send()
                    .await
                    .into_json()
                    .await;
                match result {
                    Ok(_result) => {
                        self.followed.try_set(true);
                    }
                    Err(err) => {
                        error!("follow_state: {err}");
                        self.err_general.try_set(err.to_string());
                    }
                };
            }
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_follow_state() {
    use catsquad_api::auth::create_auth_cookie_str;
    use catsquad_shared::{PostState, uuid_to_str};
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_follow_state").await;

    let (user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    let (user2, session2) = server
        .user_add_full(
            "prime2",
            "prime2@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    server
        .inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session1)),
        )
        .await;

    let followed = RwSignal::new(false);
    let follow_state = FollowState::new_with_state(
        server.client.clone(),
        user1.username.clone().into(),
        followed,
    );

    // assert err
    {
        assert!(!followed.get_untracked());
        assert!(follow_state.err_general.get_untracked().is_empty());
        follow_state.follow_toggle().await;
        assert!(!followed.get_untracked());
        assert!(!follow_state.err_general.get_untracked().is_empty());
    }

    let follow_state = FollowState::new_with_state(
        server.client.clone(),
        user2.username.clone().into(),
        followed,
    );

    // check if it clears error on success
    follow_state.err_general.set(String::from("some error"));

    // assert success
    {
        assert!(!followed.get_untracked());
        assert!(!follow_state.err_general.get_untracked().is_empty());
        follow_state.follow_toggle().await;
        assert!(followed.get_untracked());
        assert!(follow_state.err_general.get_untracked().is_empty());
        follow_state.follow_toggle().await;
        assert!(!followed.get_untracked());
        assert!(follow_state.err_general.get_untracked().is_empty());
    }
}
