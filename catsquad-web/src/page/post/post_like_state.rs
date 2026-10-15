use std::fmt::Debug;

use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use leptos::prelude::*;

#[derive(Debug, PartialEq)]
pub struct PostLikeState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub client: StoredValue<Client<TSender>, LocalStorage>,
    pub post_id: Signal<i64>,
    pub liked: RwSignal<bool>,
}

impl<TSender> Clone for PostLikeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            post_id: self.post_id.clone(),
            liked: self.liked.clone(),
        }
    }
}

impl<TSender> Copy for PostLikeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub enum LikeState {
    #[default]
    Loading,
    Liked,
    Unliked,
}

impl<TSender> PostLikeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new_with_data(
        client: Client<TSender>,
        post_id: Signal<i64>,
        liked: RwSignal<bool>,
    ) -> Self {
        Self {
            client: StoredValue::new_local(client),
            post_id,
            liked,
        }
    }

    pub async fn toggle_like(&self) {
        // let state = self.state;
        let client = self.client.get_value();
        let post_id = self.post_id.get_untracked();
        if post_id == 0 {
            return;
        }
        match self.liked.get_untracked() {
            true => {
                let result = client
                    .post_like_remove(post_id)
                    .send()
                    .await
                    .into_json()
                    .await;
                match result {
                    Ok(_result) => {
                        self.liked.set(false);
                    }
                    Err(err) => {
                        error!("use_post_like: {err}");
                    }
                };
            }
            false => {
                let result = client.post_like_add(post_id).send().await.into_json().await;
                match result {
                    Ok(_result) => {
                        self.liked.set(true);
                    }
                    Err(err) => {
                        error!("use_post_like: {err}");
                    }
                };
            }
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_like_state() {
    use catsquad_api::auth::create_auth_cookie_str;
    use catsquad_shared::{PostState, uuid_to_str};
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_post_like_state").await;

    let (_user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    let (_user2, session2) = server
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

    let post1 = server
        .client
        .post_add("", "", "")
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    server
        .client
        .post_update_state(post1.id, PostState::Active)
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    server.remove_header(header::COOKIE).await;

    server
        .inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session2)),
        )
        .await;

    let liked = RwSignal::new(false);
    let post_like_state =
        PostLikeState::new_with_data(server.client.clone(), post1.id.into(), liked);
    assert_eq!(post_like_state.liked.get_untracked(), false);
    post_like_state.toggle_like().await;
    assert_eq!(post_like_state.liked.get_untracked(), true);
    post_like_state.toggle_like().await;
    assert_eq!(post_like_state.liked.get_untracked(), false);
}
