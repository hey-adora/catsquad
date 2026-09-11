use std::fmt::Debug;

use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{
    LINK_WEB_INDEX, PostGetByKeyErr, PostRemoveErr, PostState, PostUpdateDescriptionErr,
    PostUpdateTagsErr, PostUpdateTitleErr, link_relative_img, proccess_tags,
};
use catsquad_web_utils::prelude::*;
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
    pub err_tags: RwSignal<String>,
    pub err_description: RwSignal<String>,
    pub live_description_length: RwSignal<usize, LocalStorage>,
    pub live_tags_length: RwSignal<usize>,
    pub live_title_length: RwSignal<usize, LocalStorage>,
    pub imgs_links: RwSignal<Vec<(String, f64)>, LocalStorage>,
    pub title: RwSignal<String, LocalStorage>,
    pub author_username: RwSignal<String, LocalStorage>,
    // pub author_key: RwSignal<String, LocalStorage>,
    pub author_link: RwSignal<String, LocalStorage>,
    pub tags: RwSignal<String, LocalStorage>,
    // pub tags_is_empty: RwSignal<bool, LocalStorage>,
    pub update_title_mode: RwSignal<bool, LocalStorage>,
    pub update_tags_mode: RwSignal<bool, LocalStorage>,
    pub update_description_mode: RwSignal<bool, LocalStorage>,
    // pub tags_is_e: RwSignal<String, LocalStorage>,
    pub description: RwSignal<String, LocalStorage>,
    pub post_state: RwSignal<Option<PostState>>,
    // pub description_is_empty: RwSignal<bool, LocalStorage>,
    pub likes: RwSignal<u32>,
    pub created_at: RwSignal<u64>,
    pub api_state: RwSignal<PostApiState>,
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
pub enum PostApiState {
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
            author_username: RwSignal::new_local(String::new()),
            // author_key: RwSignal::new_local(String::new()),
            author_link: RwSignal::new_local(LINK_WEB_INDEX.to_string()),
            tags: RwSignal::new_local(String::new()),
            live_description_length: RwSignal::new_local(0),
            live_tags_length: RwSignal::new(0),
            live_title_length: RwSignal::new_local(0),
            err_general: RwSignal::new_local(String::new()),
            err_title: RwSignal::new_local(String::new()),
            err_tags: RwSignal::new(String::new()),
            err_description: RwSignal::new(String::new()),
            update_title_mode: RwSignal::new_local(false),
            update_tags_mode: RwSignal::new_local(false),
            update_description_mode: RwSignal::new_local(false),
            description: RwSignal::new_local(String::new()),
            // description_is_empty: RwSignal::new_local(true),
            likes: RwSignal::new(0),
            created_at: RwSignal::new(0),
            post_state: RwSignal::new(None),
            api_state: RwSignal::new(PostApiState::Loading),
            // api,
        }
    }

    pub async fn update_description<TSender>(
        &self,
        client: &Client<TSender>,
        post_id: i64,
        new_description: impl Into<String>,
    ) -> Option<()>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let new_description = new_description.into();
        debug_data_push("post_description_mutation", new_description.clone());

        self.err_description.update(|v| v.clear());

        let result = client
            .post_update_description(post_id, new_description.clone())
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => {
                self.live_description_length.set(new_description.len());
                self.description.set(new_description);
                self.update_description_mode.set(false);
                return Some(());
            }
            Err(PostUpdateDescriptionErr::InvalidDescription(err)) => {
                self.err_description.set(err);
            }
            Err(PostUpdateDescriptionErr::PostNotFound) => {
                self.api_state.set(PostApiState::NotFound);
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
        post_id: i64,
        new_title: impl Into<String>,
    ) -> Option<()>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let new_title = new_title.into();
        // TODO test this stuff like error cleaning
        self.err_title.update(|v| v.clear());

        let result = client
            .post_update_title(post_id, new_title.clone())
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => {
                self.live_title_length.set(new_title.len());
                self.title.set(new_title);
                self.update_title_mode.set(false);
                return Some(());
            }
            Err(PostUpdateTitleErr::InvalidTitle(err)) => {
                self.err_title.set(err);
            }
            Err(PostUpdateTitleErr::PostNotFound) => {
                self.api_state.set(PostApiState::NotFound);
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
        post_id: i64,
        new_tags: impl Into<String>,
    ) -> Option<()>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let new_tags = new_tags.into();
        self.err_tags.update(|v| v.clear());

        let result = client
            .post_update_tags(post_id, new_tags.clone())
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => {
                let new_tags = proccess_tags(new_tags);
                self.live_tags_length.set(new_tags.len());
                self.tags.set(new_tags);
                self.update_tags_mode.set(false);
                return Some(());
            }
            Err(PostUpdateTagsErr::InvalidTags(err)) => {
                self.err_tags.set(err);
            }
            Err(PostUpdateTagsErr::PostNotFound) => {
                self.api_state.set(PostApiState::NotFound);
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

    pub async fn delete<TSender>(&self, client: &Client<TSender>, post_id: i64) -> Option<()>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let result = client.post_remove(post_id).send().await.into_json().await;

        match result {
            Ok(_) => {
                self.api_state.set(PostApiState::Deleted);
                return Some(());
            }
            Err(PostRemoveErr::PostNotFound) => {
                self.api_state.set(PostApiState::NotFound);
            }
            Err(err) => {
                error!("unexpected err {:#?}", { err });
            }
        }

        None
    }

    pub async fn get<TSender>(&self, client: &Client<TSender>, post_id: i64)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_id = post_id.into();
        // let (Some(username), Some(post_id)) = (param_username(), param_post.get_untracked()) else {
        //     return;
        // };

        let result = client
            .post_get_by_key(post_id)
            .send()
            .await
            .into_json()
            .await;
        match result {
            Ok(post) => {
                let post_key = post.id;
                self.live_title_length.set(post.title.len());
                self.title.set(post.title);
                self.author_username.set(post.user_username.clone());
                self.author_link.set("/404".to_string());
                // self.author_link.set(link_user(post.user.username));
                self.live_tags_length.set(post.tags.len());
                self.tags.set(post.tags);
                self.live_description_length.set(post.description.len());
                self.description.set(post.description);
                self.likes.set(post.favorites);
                self.created_at.set(post.created_at); // TODO maybe check in test
                self.imgs_links.set(
                    post.file
                        .into_iter()
                        .map(|file| {
                            (
                                link_relative_img(post_key.clone(), file.hash),
                                file.width as f64 / file.height as f64,
                            )
                        })
                        .collect(),
                );
                self.post_state.set(Some(post.state));
                self.api_state.set(PostApiState::Normal);
            }
            Err(PostGetByKeyErr::PostNotFound) => {
                self.api_state.set(PostApiState::NotFound);
                self.err_general.set(PostApiState::NotFound.to_string());
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
    use catsquad_shared::{PostState, uuid_to_str};
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
        let (_owner, app, post_id) =
            post_setup("hook_post_api_update_description", "title", "0", "").await;

        // testing normal
        let post_api = PostApi::new();
        post_api.get(&app.client, post_id).await;
        assert_eq!(post_api.live_description_length.get_untracked(), 1);
        assert_eq!(post_api.description.get_untracked(), "0");
        post_api.update_description_mode.set(true);
        assert!(post_api.err_description.get_untracked().is_empty());

        post_api
            .update_description(&app.client, post_id, "22")
            .await;
        assert_eq!(post_api.live_description_length.get_untracked(), 2);
        assert_eq!(post_api.description.get_untracked(), "22");
        assert_eq!(post_api.update_tags_mode.get_untracked(), false);
        assert!(post_api.err_description.get_untracked().is_empty());

        let post_api = PostApi::new();
        post_api.get(&app.client, post_id).await;
        assert_eq!(post_api.live_description_length.get_untracked(), 2);
        assert_eq!(post_api.description.get_untracked(), "22");

        post_api.delete(&app.client, post_id).await;

        post_api.update_description(&app.client, post_id, "2").await;
        assert_eq!(post_api.live_description_length.get_untracked(), 2);
        assert!(!post_api.err_general.get_untracked().is_empty());

        // let items = gallery_api.items.get_untracked();
        // assert_eq!(items.len(), 1);
    }

    pub async fn post_setup(
        db: &str,
        title: impl Into<String>,
        description: impl Into<String>,
        tags: impl Into<String>,
    ) -> (Owner, TestServer, i64) {
        init_log();
        let owner = init_owner();
        let app = TestServer::new(0, db).await;
        let (user1, session_token1) = app
            .user_add_full("hey", "hey@heyadora.com", "pas$worFd123456789")
            .await;
        app.inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session_token1)),
        )
        .await;

        let post_id = {
            let post = app
                .client
                .post_add(title, description, tags)
                .send()
                .await
                .into_json()
                .await
                .unwrap();
            app.client
                .post_update_state(post.id, PostState::Active)
                .send()
                .await
                .into_json()
                .await
                .unwrap();
            // app.state.set_time(1).await;
            post.id
        };

        (owner, app, post_id)
    }

    #[tokio::test]
    pub async fn hook_post_api_update_title() {
        // let _owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let (_owner, app, post_id) =
            post_setup("hook_post_api_update_title", "title", "", "").await;
        // testing err

        let post_api = PostApi::new();
        post_api.get(&app.client, 0).await;
        assert!(!post_api.err_general.get_untracked().is_empty());
        assert_eq!(post_api.title.get_untracked(), "");
        assert_eq!(post_api.live_title_length.get_untracked(), 0);

        // testing normal
        let post_api = PostApi::new();
        post_api.get(&app.client, post_id).await;
        assert_eq!(post_api.title.get_untracked(), "title");
        post_api.update_title_mode.set(true);
        assert!(post_api.err_title.get_untracked().is_empty());
        assert_eq!(post_api.live_title_length.get_untracked(), 5);

        post_api.update_title(&app.client, post_id, "one").await;
        assert_eq!(post_api.title.get_untracked(), "one");
        assert_eq!(post_api.update_title_mode.get_untracked(), false);
        assert_eq!(post_api.live_title_length.get_untracked(), 3);

        let post_api = PostApi::new();
        post_api.get(&app.client, post_id).await;
        assert_eq!(post_api.title.get_untracked(), "one");
        assert_eq!(post_api.live_title_length.get_untracked(), 3);

        // let items = gallery_api.items.get_untracked();
        // assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    pub async fn hook_post_api_update_tags() {
        let (_owner, app, post_id) = post_setup("hook_post_api_update_tags", "title", "", "").await;

        // testing err
        let post_api = PostApi::new();
        post_api.get(&app.client, 0).await;
        assert!(!post_api.err_general.get_untracked().is_empty());
        assert_eq!(post_api.tags.get_untracked(), "");
        assert_eq!(post_api.live_tags_length.get_untracked(), 0);

        // testing normal
        let post_api = PostApi::new();
        post_api.get(&app.client, post_id).await;
        assert_eq!(post_api.tags.get_untracked(), "");
        post_api.update_tags_mode.set(true);
        assert!(post_api.err_tags.get_untracked().is_empty());
        assert_eq!(post_api.live_tags_length.get_untracked(), 0);

        post_api.update_tags(&app.client, post_id, "oNe     ").await;
        assert_eq!(post_api.tags.get_untracked(), " one ");
        assert_eq!(post_api.update_tags_mode.get_untracked(), false);
        assert_eq!(post_api.live_tags_length.get_untracked(), 5);

        let post_api = PostApi::new();
        post_api.get(&app.client, post_id).await;
        assert_eq!(post_api.tags.get_untracked(), " one ");
        assert_eq!(post_api.live_tags_length.get_untracked(), 5);

        // let items = gallery_api.items.get_untracked();
        // assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    pub async fn hook_post_api_delete() {
        let (_owner, app, post_id) = post_setup("hook_post_api_delete", "title", "", "").await;

        let post_api = PostApi::new();
        post_api.get(&app.client, post_id).await;

        let result = post_api.delete(&app.client, post_id).await;
        assert!(result.is_some());

        let post_all = app.state.db.post_get_all().await.unwrap();
        assert_eq!(post_all.len(), 0);
    }

    #[tokio::test]
    pub async fn hook_post_api_post() {
        let (_owner, app, post_id) = post_setup("hook_post_api_post", "title", "", "").await;

        let post_api = PostApi::new();
        post_api.get(&app.client, post_id).await;
        assert_eq!(post_api.title.get_untracked(), "title");
        assert_ne!(post_api.author_username.get_untracked(), "");
    }
}
