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
    pub post_key: StoredValue<String>,
    pub state: RwSignal<LikeState>,
}

impl<TSender> Clone for PostLikeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            post_key: self.post_key.clone(),
            state: self.state.clone(),
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
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            client: StoredValue::new_local(client),
            post_key: StoredValue::new(String::new()),
            state: RwSignal::new(LikeState::default()),
        }
    }

    pub async fn init(&self, post_key: impl Into<String>) {
        let client = self.client.get_value();
        let state = self.state;
        let post_key = post_key.into();
        let result = client
            .post_like_get_by_post(&post_key)
            .send()
            .await
            .into_json()
            .await;
        match result {
            Ok(liked) => {
                self.post_key.set_value(post_key);
                if liked {
                    state.set(LikeState::Liked);
                } else {
                    state.set(LikeState::Unliked);
                }
            }
            Err(err) => {
                error!("use_post_like: {err}");
                state.set(LikeState::Unliked);
            }
        };
    }

    pub async fn toggle_like(&self) {
        let state = self.state;
        let client = self.client.get_value();
        let post_key = self.post_key.get_value();
        if post_key.is_empty() {
            return;
        }
        match state.get_untracked() {
            LikeState::Loading => {
                //
            }
            LikeState::Liked => {
                let result = client
                    .post_like_remove(post_key)
                    .send()
                    .await
                    .into_json()
                    .await;
                match result {
                    Ok(_result) => {
                        state.set(LikeState::Unliked);
                    }
                    Err(err) => {
                        error!("use_post_like: {err}");
                        state.set(LikeState::Unliked);
                    }
                };
            }
            LikeState::Unliked => {
                let result = client
                    .post_like_add(post_key)
                    .send()
                    .await
                    .into_json()
                    .await;
                match result {
                    Ok(_result) => {
                        state.set(LikeState::Liked);
                    }
                    Err(err) => {
                        error!("use_post_like: {err}");
                        state.set(LikeState::Unliked);
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
    use catsquad_shared::PostState;
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;

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
        .inject_header(header::COOKIE, create_auth_cookie_str(session1.clone()))
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
        .post_update_state(post1.key.clone(), PostState::Active)
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    server.remove_header(header::COOKIE).await;

    server
        .inject_header(header::COOKIE, create_auth_cookie_str(session2.clone()))
        .await;

    let post_like_state = PostLikeState::new(server.client.clone());
    assert_eq!(post_like_state.state.get_untracked(), LikeState::Loading);
    post_like_state.init(post1.key.clone()).await;
    assert_eq!(post_like_state.state.get_untracked(), LikeState::Unliked);
    post_like_state.toggle_like().await;
    assert_eq!(post_like_state.state.get_untracked(), LikeState::Liked);
    post_like_state.toggle_like().await;
    assert_eq!(post_like_state.state.get_untracked(), LikeState::Unliked);
}

// #[derive(Clone, Copy)]
// pub struct PostLike {
//     pub stage: StoredValue<Box<dyn Fn() -> LikeState + Sync + Send + 'static>>,
//     pub on_like: StoredValue<Box<dyn Fn() + Sync + Send + 'static>>,
// }

// pub fn use_post_like(post_key: Memo<Option<String>>) -> PostLike {
//     // let api = ApiWeb::new();
//     let spawner = Spawner::new();
//     let stage = RwSignal::new(LikeState::Loading);
//     let stage_view = move || {
//         if spawner.is_busy.get() {
//             LikeState::Loading
//         } else {
//             stage.get()
//         }
//     };

//     Effect::new({
//         let post_key = post_key.clone();
//         move || {
//             let Some(post_key) = post_key.get() else {
//                 return;
//             };
//             let client = create_client();
//             spawner.spawn(async move {
//                 let result = client
//                     .post_like_get_by_post(post_key)
//                     .send()
//                     .await
//                     .into_json()
//                     .await;
//                 match result {
//                     Ok(liked) => {
//                         if liked {
//                             stage.set(LikeState::Liked);
//                         } else {
//                             stage.set(LikeState::Unliked);
//                         }
//                     }
//                     Err(err) => {
//                         error!("use_post_like: {err}");
//                         stage.set(LikeState::Unliked);
//                     }
//                 };
//             });
//         }
//     });

//     let on_like = move || {
//         let Some(post_id) = post_key.get() else {
//             return;
//         };
//         match stage.get_untracked() {
//             LikeState::Loading => {
//                 //
//             }
//             LikeState::Liked => {
//                 spawner.spawn(async move {
//                     let client = create_client();
//                     let result = client
//                         .post_like_remove(post_id)
//                         .send()
//                         .await
//                         .into_json()
//                         .await;
//                     match result {
//                         Ok(_result) => {
//                             stage.set(LikeState::Unliked);
//                         }
//                         Err(err) => {
//                             error!("use_post_like: {err}");
//                             stage.set(LikeState::Unliked);
//                         }
//                     };
//                 });
//             }
//             LikeState::Unliked => {
//                 spawner.spawn(async move {
//                     let client = create_client();
//                     let result = client.post_like_add(post_id).send().await.into_json().await;
//                     match result {
//                         Ok(_result) => {
//                             stage.set(LikeState::Liked);
//                         }
//                         Err(err) => {
//                             error!("use_post_like: {err}");
//                             stage.set(LikeState::Unliked);
//                         }
//                     };
//                 });
//             }
//         }
//     };

//     PostLike {
//         stage: StoredValue::new(Box::new(stage_view)),
//         on_like: StoredValue::new(Box::new(on_like)),
//     }
// }
