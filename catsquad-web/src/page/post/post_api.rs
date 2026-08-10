use std::fmt::Debug;

use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{
    LINK_WEB_INDEX, PostGetByKeyErr, PostRemoveErr, PostUpdateDescriptionErr, PostUpdateTagsErr,
    PostUpdateTitleErr, link_relative_img,
};
use leptos::prelude::*;
// use crate::{
//     api::{Api, Server404Err, ServerErr, ServerUpdatePostDescriptionErr},
//     path::{link_home, link_img, link_user},
// };
// use tracing::{error, info, trace, warn};

// pub struct PostApi<TSender>
// where
//     TSender: Sender + Debug + Clone,
//     TSender::TResponse: Response + Debug,
#[derive(Clone, Copy)]
pub struct PostApi {
    // ui
    // pub items: RwSignal<Vec<Img>, LocalStorage>,
    pub err_general: RwSignal<String, LocalStorage>,
    pub err_title: RwSignal<String, LocalStorage>,
    pub err_tags: RwSignal<String, LocalStorage>,
    pub err_description: RwSignal<String, LocalStorage>,
    pub live_description_length: RwSignal<usize, LocalStorage>,
    pub live_tags_length: RwSignal<usize, LocalStorage>,
    pub live_title_length: RwSignal<usize, LocalStorage>,
    pub imgs_links: RwSignal<Vec<(String, f64)>, LocalStorage>,
    pub title: RwSignal<String, LocalStorage>,
    pub author: RwSignal<String, LocalStorage>,
    pub author_link: RwSignal<String, LocalStorage>,
    pub tags: RwSignal<String, LocalStorage>,
    // pub tags_is_empty: RwSignal<bool, LocalStorage>,
    pub update_title_mode: RwSignal<bool, LocalStorage>,
    pub update_tags_mode: RwSignal<bool, LocalStorage>,
    pub update_description_mode: RwSignal<bool, LocalStorage>,
    // pub tags_is_e: RwSignal<String, LocalStorage>,
    pub description: RwSignal<String, LocalStorage>,
    // pub description_is_empty: RwSignal<bool, LocalStorage>,
    pub favorites: RwSignal<u64, LocalStorage>,
    pub post_state: RwSignal<PostState, LocalStorage>,
    // pub api: Client<TSender>,
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum PostState {
    #[default]
    Loading,
    Normal,
    NotFound,
    Deleted,
}

impl PostApi {
    pub fn new() -> Self {
        Self {
            // items: RwSignal::new_local(Vec::new()),
            imgs_links: RwSignal::new_local(Vec::<(String, f64)>::new()),
            title: RwSignal::new_local(String::new()),
            author: RwSignal::new_local(String::new()),
            author_link: RwSignal::new_local(LINK_WEB_INDEX.to_string()),
            tags: RwSignal::new_local(String::new()),
            live_description_length: RwSignal::new_local(0),
            live_tags_length: RwSignal::new_local(0),
            live_title_length: RwSignal::new_local(0),
            err_general: RwSignal::new_local(String::new()),
            err_title: RwSignal::new_local(String::new()),
            err_tags: RwSignal::new_local(String::new()),
            err_description: RwSignal::new_local(String::new()),
            update_title_mode: RwSignal::new_local(false),
            update_tags_mode: RwSignal::new_local(false),
            update_description_mode: RwSignal::new_local(false),
            description: RwSignal::new_local(String::new()),
            // description_is_empty: RwSignal::new_local(true),
            favorites: RwSignal::new_local(0_u64),
            post_state: RwSignal::new_local(PostState::Loading),
            // api,
        }
    }

    pub async fn update_description<TSender>(
        &self,
        client: &Client<TSender>,
        post_key: impl Into<String>,
        description: impl Into<String>,
    ) -> Option<()>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        self.err_description.update(|v| v.clear());

        let result = client
            .post_update_description(post_key, description)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {
                self.live_description_length.set(v.description.len());
                self.description.set(v.description);
                self.update_description_mode.set(false);
                return Some(());
            }
            Err(PostUpdateDescriptionErr::InvalidDescription(err)) => {
                self.err_description.set(err);
            }
            Err(PostUpdateDescriptionErr::PostNotFound) => {
                self.post_state.set(PostState::NotFound);
                self.err_general.set("post not found".to_string());
            }
            Err(err) => {
                let err = format!("unexpected err {:#?}", { err });
                error!(err);
                self.err_description.set(err);
            }
        }

        None
    }

    pub async fn update_title<TSender>(
        &self,
        client: &Client<TSender>,
        post_key: impl Into<String>,
        title: impl Into<String>,
    ) -> Option<()>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        // TODO test this stuff like error cleaning
        self.err_title.update(|v| v.clear());

        let result = client
            .post_update_title(post_key, title)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {
                self.live_title_length.set(v.title.len());
                self.title.set(v.title);
                self.update_title_mode.set(false);
                return Some(());
            }
            Err(PostUpdateTitleErr::InvalidTitle(err)) => {
                self.err_title.set(err);
            }
            Err(PostUpdateTitleErr::PostNotFound) => {
                self.post_state.set(PostState::NotFound);
                self.err_general.set("post not found".to_string());
            }
            Err(err) => {
                let err = format!("unexpected err {:#?}", { err });
                error!(err);
                self.err_title.set(err);
            }
        }

        None
    }

    pub async fn update_tags<TSender>(
        &self,
        client: &Client<TSender>,
        post_key: impl Into<String>,
        tags: impl Into<String>,
    ) -> Option<()>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        self.err_tags.update(|v| v.clear());

        let result = client
            .post_update_tags(post_key, tags)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {
                self.live_tags_length.set(v.tags.len());
                self.tags.set(v.tags);
                self.update_tags_mode.set(false);
                return Some(());
            }
            Err(PostUpdateTagsErr::InvalidTags(err)) => {
                self.err_tags.set(err);
            }
            Err(PostUpdateTagsErr::PostNotFound) => {
                self.post_state.set(PostState::NotFound);
                self.err_general.set("post not found".to_string());
            }
            Err(err) => {
                let err = format!("unexpected err {:#?}", { err });
                error!(err);
                self.err_tags.set(err);
            }
        }

        None
    }

    pub async fn delete<TSender>(
        &self,
        client: &Client<TSender>,
        post_id: impl Into<String>,
    ) -> Option<()>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_id = post_id.into();
        let result = client.post_remove(post_id).send().await.into_json().await;

        match result {
            Ok(_) => {
                self.post_state.set(PostState::Deleted);
                return Some(());
            }
            Err(PostRemoveErr::PostNotFound) => {
                self.post_state.set(PostState::NotFound);
            }
            Err(err) => {
                error!("unexpected err {:#?}", { err });
            }
        }

        None
    }

    pub async fn get<TSender>(&self, client: &Client<TSender>, post_key: impl Into<String>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_key = post_key.into();
        // let (Some(username), Some(post_id)) = (param_username(), param_post.get_untracked()) else {
        //     return;
        // };

        let result = client
            .post_get_by_key(post_key)
            .send()
            .await
            .into_json()
            .await;
        match result {
            Ok(post) => {
                self.live_title_length.set(post.title.len());
                self.title.set(post.title);
                self.author.set(post.user.username.clone());
                self.author_link.set("/404".to_string());
                // self.author_link.set(link_user(post.user.username));
                self.live_tags_length.set(post.tags.len());
                self.tags.set(post.tags);
                self.live_description_length.set(post.description.len());
                self.description.set(post.description);
                // if post.description.is_empty() {
                //     self.description.set("No description.".to_string());
                //     // self.description_is_empty.set(true);
                // } else {
                //     self.description.set(post.description);
                //     // self.description_is_empty.set(false);
                // }

                self.favorites.set(post.favorites);
                self.imgs_links.set(
                    post.file
                        .into_iter()
                        .map(|file| {
                            (
                                link_relative_img(file.hash, file.extension),
                                file.width as f64 / file.height as f64,
                            )
                        })
                        .collect(),
                );
                self.post_state.set(PostState::Normal);
            }
            Err(PostGetByKeyErr::PostNotFound) => {
                self.post_state.set(PostState::NotFound);
                self.err_general.set(PostState::NotFound.to_string());
            }
            Err(err) => {
                let err = format!("unexpected err {:#?}", { err });
                error!(err);
                self.err_general.set(err);
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use catsquad_api::{TestServer, auth::create_auth_cookie_str};
    use catsquad_shared::PostState;
    use http::header;
    // use crate::{
    //     api::{
    //         Order, ServerReqImg, TimeRange, shared::post_comment::UserPostComment,
    //         tests::ApiTestApp,
    //     },
    //     view::{
    //         app::hook::{
    //             api_gallery::{GalleryApi, GalleryContainerSize},
    //             api_post::PostApi,
    //             api_post_comments::{CommentKind, CommentKind2, CommentsApi, CommentsApi2},
    //             use_scroll_correction::ScrollCorrection,
    //         },
    //         logger,
    //         toolbox::prelude::*,
    //     },
    // };
    // use hydration_context::HydrateSharedContext;
    use leptos::prelude::*;
    use std::sync::Arc;
    // use surrealdb::types::ToSql;
    use catsquad_log::prelude::*;
    use tokio::process::Command;

    use crate::{init_owner, page::post::post_api::PostApi};
    // use tracing::{debug, trace};

    // use crate::init_test_log;

    #[tokio::test]
    pub async fn hook_post_api_update_description() {
        let (_owner, app, post_key) = post_setup("title", "0", "").await;

        // testing normal
        let post_api = PostApi::new();
        post_api.get(&app.client, &post_key).await;
        assert_eq!(post_api.live_description_length.get_untracked(), 1);
        assert_eq!(post_api.description.get_untracked(), "0");
        post_api.update_description_mode.set(true);
        assert!(post_api.err_description.get_untracked().is_empty());

        post_api
            .update_description(&app.client, &post_key, "22")
            .await;
        assert_eq!(post_api.live_description_length.get_untracked(), 2);
        assert_eq!(post_api.description.get_untracked(), "22");
        assert_eq!(post_api.update_tags_mode.get_untracked(), false);
        assert!(post_api.err_description.get_untracked().is_empty());

        let post_api = PostApi::new();
        post_api.get(&app.client, &post_key).await;
        assert_eq!(post_api.live_description_length.get_untracked(), 2);
        assert_eq!(post_api.description.get_untracked(), "22");

        post_api.delete(&app.client, &post_key).await;

        post_api
            .update_description(&app.client, &post_key, "2")
            .await;
        assert_eq!(post_api.live_description_length.get_untracked(), 2);
        assert!(!post_api.err_general.get_untracked().is_empty());

        // let items = gallery_api.items.get_untracked();
        // assert_eq!(items.len(), 1);
    }

    pub async fn post_setup(
        title: impl Into<String>,
        description: impl Into<String>,
        tags: impl Into<String>,
    ) -> (Owner, TestServer, String) {
        init_log();
        let owner = init_owner();
        let app = TestServer::new().await;
        let (user1, session_key1) = app
            .user_add_full("hey", "hey@heyadora.com", "pas$worFd123456789")
            .await;
        app.inject_header(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
            .await;

        let post_key = {
            let post = app
                .client
                .post_add(title, description, tags)
                .send()
                .await
                .into_json()
                .await
                .unwrap();
            app.client
                .post_update_state(&post.key, PostState::Active)
                .send()
                .await
                .into_json()
                .await
                .unwrap();
            // app.state.set_time(1).await;
            post.key.clone()
        };

        (owner, app, post_key)
    }

    #[tokio::test]
    pub async fn hook_post_api_update_title() {
        // let _owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let (_owner, app, post_key) = post_setup("title", "", "").await;
        // testing err
        let post_api = PostApi::new();
        post_api.get(&app.client, "invalid").await;
        assert!(!post_api.err_general.get_untracked().is_empty());
        assert_eq!(post_api.title.get_untracked(), "");
        assert_eq!(post_api.live_title_length.get_untracked(), 0);

        // testing normal
        let post_api = PostApi::new();
        post_api.get(&app.client, &post_key).await;
        assert_eq!(post_api.title.get_untracked(), "title");
        post_api.update_title_mode.set(true);
        assert!(post_api.err_title.get_untracked().is_empty());
        assert_eq!(post_api.live_title_length.get_untracked(), 5);

        post_api.update_title(&app.client, &post_key, "one").await;
        assert_eq!(post_api.title.get_untracked(), "one");
        assert_eq!(post_api.update_title_mode.get_untracked(), false);
        assert_eq!(post_api.live_title_length.get_untracked(), 3);

        let post_api = PostApi::new();
        post_api.get(&app.client, &post_key).await;
        assert_eq!(post_api.title.get_untracked(), "one");
        assert_eq!(post_api.live_title_length.get_untracked(), 3);

        // let items = gallery_api.items.get_untracked();
        // assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    pub async fn hook_post_api_update_tags() {
        let (_owner, app, post_key) = post_setup("title", "", "").await;

        // testing err
        let post_api = PostApi::new();
        post_api.get(&app.client, "invalid").await;
        assert!(!post_api.err_general.get_untracked().is_empty());
        assert_eq!(post_api.tags.get_untracked(), "");
        assert_eq!(post_api.live_tags_length.get_untracked(), 0);

        // testing normal
        let post_api = PostApi::new();
        post_api.get(&app.client, &post_key).await;
        assert_eq!(post_api.tags.get_untracked(), "");
        post_api.update_tags_mode.set(true);
        assert!(post_api.err_tags.get_untracked().is_empty());
        assert_eq!(post_api.live_tags_length.get_untracked(), 0);

        post_api.update_tags(&app.client, &post_key, "one").await;
        assert_eq!(post_api.tags.get_untracked(), "one");
        assert_eq!(post_api.update_tags_mode.get_untracked(), false);
        assert_eq!(post_api.live_tags_length.get_untracked(), 3);

        let post_api = PostApi::new();
        post_api.get(&app.client, &post_key).await;
        assert_eq!(post_api.tags.get_untracked(), "one");
        assert_eq!(post_api.live_tags_length.get_untracked(), 3);

        // let items = gallery_api.items.get_untracked();
        // assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    pub async fn hook_post_api_delete() {
        let (_owner, app, post_key) = post_setup("title", "", "").await;

        let post_api = PostApi::new();
        post_api.get(&app.client, &post_key).await;

        let result = post_api.delete(&app.client, &post_key).await;
        assert!(result.is_some());

        let post_all = app.state.db.post_get_all().await.unwrap();
        assert_eq!(post_all.len(), 0);
    }

    #[tokio::test]
    pub async fn hook_post_api_post() {
        let (_owner, app, post_key) = post_setup("title", "", "").await;

        let post_api = PostApi::new();
        post_api.get(&app.client, &post_key).await;
        assert_eq!(post_api.title.get_untracked(), "title");
    }
}
