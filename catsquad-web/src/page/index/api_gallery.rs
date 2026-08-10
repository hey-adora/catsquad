use std::fmt::Debug;

// use crate::{
//     api::{
//         Api, ApiWeb, Order, ServerErr, ServerReqImg, ServerRes, TimeRange, UserPost,
//         shared::post_comment::UserPostComment,
//     },
//     view::{
//         app::{
//             components::gallery::{Img, add_imgs_to_bottom, add_imgs_to_top},
//             hook::{
//                 use_future::FutureFn, use_infinite_scroll_basic::InfiniteBasic,
//                 use_infinite_scroll_fn::InfiniteItem, use_scroll_correction::ScrollCorrection,
//             },
//         },
//         toolbox::prelude::*,
//     },
// };
use crate::hook::ScrollCorrection;
use catsquad_client::{Client, Response, Sender};
use catsquad_shared::{Order, TimeRange};
use leptos::{
    html::{ElementType, Textarea},
    prelude::*,
};

// use tracing::{debug, error, trace, warn};
use super::component_gallery::{Img, add_imgs_to_bottom, add_imgs_to_top};
use catsquad_log::prelude::*;
use catsquad_web_utils::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlTextAreaElement, MutationObserver, MutationRecord};

#[derive(Clone, Copy, Default, Debug)]
pub struct GalleryContainerSize {
    pub width: u32,
    pub height: f64,
    pub row_height: u32,
}

#[derive(Clone, Copy)]
pub struct GalleryApi {
    // ui
    // pub items: RwSignal<Vec<Img>, LocalStorage>,
    pub items: StoreSignal<Vec<Img>>,
    pub scroll_correction_handle: ScrollCorrection,
    // params
    // pub api_top: API,
    // pub api_btm: API,
}

// TODO maybe it would be better design to have these as seperate functions, without struct
// abstraction
impl GalleryApi {
    pub fn new(scroll_correction_handle: ScrollCorrection) -> Self {
        let items = StoreSignal::new_with_formmater(true, "gallery_api_items", Vec::new(), |v| {
            serde_json::to_string(v).unwrap_or_else(|e| e.to_string())
        });
        Self {
            scroll_correction_handle,
            items,
            // items: RwSignal::new_local(Vec::new()),
            // api_top,
            // api_btm,
        }
    }

    // pub fn observe_only(&self, size: PostContainerSize) {
    //     self.size.set_value(size);
    // }

    pub async fn post<TSender>(
        &self,
        client: &Client<TSender>,
        size: GalleryContainerSize,
        title: impl Into<String>,
        description: impl Into<String>,
        tags: impl Into<String>,
        // files: Vec<ServerReqImg>,
    ) -> f64
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let items = self.items;

        let result = client
            .post_add(title, description, tags)
            .send()
            .await
            .into_json()
            .await;
        // let result = self
        //     .api_top
        //     .add_post(title, description, tags)
        //     .send_native()
        //     .await;

        match result {
            Ok(post) => {
                let new_img = Img::from(post);
                let new_imgs = Vec::from([new_img]);
                let old_imgs = items.get_untracked();

                trace!("CAN I MAKE THIS OR NOT");
                let (resized_imgs, scroll_by) = add_imgs_to_bottom(
                    old_imgs,
                    new_imgs,
                    size.width,
                    size.height,
                    size.row_height,
                );
                items.set(resized_imgs);
                return scroll_by;
            }
            Ok(err) => {
                let err = format!("post comments basic: unexpected res: {err:?}");
                error!(err);
                // self.err_fetch.set(err);
            }
            Err(err) => {
                let err = format!("post comments basic: {err}");
                error!(err);
                // self.err_fetch.set(err);
            }
        };
        0.0
    }

    pub async fn fetch<TSender>(
        self,
        client: &Client<TSender>,
        limit: usize,
        size: GalleryContainerSize,
        time: u128,
        range: TimeRange,
        order: Order,
        reverse: bool,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let tags = tags.into();
        let username = username.into();
        let items = self.items;
        let scroll_correction = self.scroll_correction_handle;

        let is_bottom = match range {
            TimeRange::None => true,
            TimeRange::Less => true,
            TimeRange::LessOrEqual => true,
            TimeRange::More => false,
            TimeRange::MoreOrEqual => false,
        };

        // let api = if is_bottom {
        //     &self.api_top
        // } else {
        //     &self.api_btm
        // };

        let result = client
            .post_search(tags, username, time, limit, range, order)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(mut posts) => {
                if reverse {
                    posts.reverse();
                }

                let new_imgs = posts.into_iter().map(Img::from).collect::<Vec<Img>>();
                let old_imgs = items.get_untracked();

                let (resized_imgs, scroll_by) = if is_bottom {
                    add_imgs_to_bottom(old_imgs, new_imgs, size.width, size.height, size.row_height)
                } else {
                    add_imgs_to_top(old_imgs, new_imgs, size.width, size.height, size.row_height)
                };
                scroll_correction.update();
                items.set(resized_imgs);

                return scroll_by;
            }
            Ok(err) => {
                let err = format!("post comments basic: unexpected res: {err:?}");
                error!(err);
                // self.err_fetch.set(err);
            }
            Err(err) => {
                let err = format!("post comments basic: {err}");
                error!(err);
                // self.err_fetch.set(err);
            }
        };

        0.0
    }

    pub async fn fetch_btm<TSender>(
        self,
        client: &Client<TSender>,
        limit: usize,
        size: GalleryContainerSize,
        current_time: u128,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let (time, range) = self
            .items
            .with_untracked(|v| v.last().map(|v| v.created_at))
            .map(|time| (time, TimeRange::Less))
            .unwrap_or((current_time, TimeRange::LessOrEqual));

        self.fetch(
            client,
            limit,
            size,
            time,
            range,
            Order::ThreeTwoOne,
            false,
            tags,
            username,
        )
        .await
    }

    pub async fn fetch_top<TSender>(
        self,
        client: &Client<TSender>,
        limit: usize,
        size: GalleryContainerSize,
        current_time: u128,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let (time, range) = self
            .items
            .with_untracked(|v| v.first().map(|v| v.created_at))
            .map(|time| (time, TimeRange::More))
            .unwrap_or((current_time, TimeRange::MoreOrEqual));
        debug!("time range picked: {time} {range:?}");

        self.fetch(
            client,
            limit,
            size,
            time,
            range,
            Order::OneTwoThree,
            true,
            tags,
            username,
        )
        .await
    }

    pub async fn fetch_btm_or_top<TSender>(
        self,
        client: &Client<TSender>,
        is_bottom: bool,
        limit: usize,
        size: GalleryContainerSize,
        current_time: u128,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        if is_bottom {
            self.fetch_btm(client, limit, size, current_time, tags, username)
                .await
        } else {
            self.fetch_top(client, limit, size, current_time, tags, username)
                .await
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.with_untracked(|v| v.is_empty())
    }

    pub fn reset(&self) {
        self.items.set(Vec::new());
    }
}

#[cfg(test)]
pub mod tests {
    use catsquad_api::{TestServer, auth::create_auth_cookie_str};
    // use crate::{
    //     api::{ServerReqImg, tests::ApiTestApp},
    //     view::app::hook::{
    //         api_gallery::{GalleryApi, GalleryContainerSize},
    //         use_scroll_correction::ScrollCorrection,
    //     },
    // };
    // use hydration_context::HydrateSharedContext;
    use catsquad_log::prelude::*;
    use http::header;
    use leptos::prelude::*;
    use std::sync::Arc;

    use crate::{
        hook::ScrollCorrection,
        init_owner,
        page::index::api_gallery::{GalleryApi, GalleryContainerSize},
    };
    // use tracing::{debug, trace};

    // use crate::init_test_log;

    #[tokio::test]
    pub async fn hook_gallery_api_post() {
        // println!("hello");
        // init_test_log();
        //
        init_log();
        let _owner = init_owner();
        // let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let scroll_corerction = ScrollCorrection::new();
        let mut app = TestServer::new().await;

        let (user1, session_key1) = app
            .user_add_full("hey", "hey@heyadora.com", "pas$word123456789")
            .await;

        let (user2, session_key2) = app
            .user_add_full("hey2", "hey2@heyadora.com", "pas$word123456789")
            .await;

        app.inject_header(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
            .await;
        // app.api.auth_token_overwrite = auth_token.clone();

        let post_api = GalleryApi::new(scroll_corerction.clone());
        let size = GalleryContainerSize {
            width: 100,
            height: 100.0,
            row_height: 50,
        };
        // post_api.observe_only(PostContainerSize {
        //     width: 100,
        //     height: 100.0,
        //     row_height: 50,
        // });

        app.state.set_time(1).await;
        post_api
            .post(
                &app.client,
                size,
                "title1",
                "0",
                "",
                // vec![create_img_req("1", 50, 50).await],
            )
            .await;
        app.state.set_time(2).await;
        post_api
            .post(
                &app.client,
                size,
                "title2",
                "0",
                "",
                // vec![create_img_req("2", 50, 50).await],
            )
            .await;
        app.state.set_time(3).await;
        post_api
            .post(
                &app.client,
                size,
                "title3",
                "0",
                "",
                // vec![create_img_req("3", 50, 50).await],
            )
            .await;
        let items = post_api.items.get_untracked();
        trace!("aaaaa {items:#?}");
        assert_eq!(items.len(), 3);

        app.state.set_time(4).await;
        let post_api2 = GalleryApi::new(scroll_corerction.clone());
        post_api2.fetch_btm(&app.client, 10, size, 4, "", "").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 3);

        let post_api3 = GalleryApi::new(scroll_corerction.clone());
        post_api3.fetch_btm(&app.client, 2, size, 4, "", "").await;
        let items = post_api3.items.get_untracked();
        assert_eq!(items.len(), 2);
        post_api3.fetch_btm(&app.client, 2, size, 4, "", "").await;
        let items = post_api3.items.get_untracked();
        assert_eq!(items.len(), 3);

        let post_api = GalleryApi::new(scroll_corerction.clone());
        post_api.fetch_top(&app.client, 2, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 0);

        let post_api = GalleryApi::new(scroll_corerction.clone());
        post_api.fetch_top(&app.client, 2, size, 0, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].created_at, 2);
        assert_eq!(items[1].created_at, 1);
        post_api.fetch_top(&app.client, 2, size, 0, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].created_at, 3);
        assert_eq!(items[1].created_at, 2);
        assert_eq!(items[2].created_at, 1);

        let post_api = GalleryApi::new(scroll_corerction.clone());
        post_api.fetch_btm(&app.client, 3, size, 3, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].created_at, 3);
        assert_eq!(items[1].created_at, 2);
        assert_eq!(items[2].created_at, 1);
        post_api.items.update_untracked(|v| {
            v.remove(0);
            v.remove(0);
        });
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].created_at, 1);
        post_api.fetch_top(&app.client, 1, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].created_at, 2);
        assert_eq!(items[1].created_at, 1);
        post_api.fetch_top(&app.client, 1, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].created_at, 3);
        assert_eq!(items[1].created_at, 2);
        assert_eq!(items[2].created_at, 1);

        // let items = post_api.items.get_untracked();
        // assert_eq!(items.len(), 2);
        // assert_eq!(items[0].created_at, 2);
        // post_api.fetch_top(1, size, 4, "", "").await;
        // let items = post_api.items.get_untracked();
        // assert_eq!(items.len(), 3);
        // assert_eq!(items[0].created_at, 1);

        // post_api.fetch_top(1, size, 4, "", "").await;
        // let items = post_api.items.get_untracked();
        // assert_eq!(items.len(), 2);

        let post_api = GalleryApi::new(scroll_corerction.clone());
        post_api.fetch_btm(&app.client, 50, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        trace!("ITEMS1: {items:#?}");
        assert_eq!(items.len(), 3);
        post_api.fetch_top(&app.client, 50, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        trace!("ITEMS2: {items:#?}");
        assert_eq!(items.len(), 3);

        app.remove_header(header::COOKIE).await;
        app.inject_header(header::COOKIE, create_auth_cookie_str(session_key2.clone()))
            .await;

        let post_api = GalleryApi::new(scroll_corerction.clone());

        app.state.set_time(5).await;
        post_api
            .post(
                &app.client,
                size,
                "title1",
                "0",
                "one two three",
                // vec![create_img_req("1", 50, 50).await],
            )
            .await;
        app.state.set_time(6).await;
        post_api
            .post(
                &app.client,
                size,
                "title2",
                "0",
                "one two",
                // vec![create_img_req("2", 50, 50).await],
            )
            .await;
        app.state.set_time(7).await;
        post_api
            .post(
                &app.client,
                size,
                "title3",
                "0",
                "one",
                // vec![create_img_req("3", 50, 50).await],
            )
            .await;

        app.state.set_time(8).await;
        let post_api2 = GalleryApi::new(scroll_corerction.clone());
        post_api2
            .fetch_btm(&app.client, 2, size, 8, "one", "hey2")
            .await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].created_at, 7);
        assert_eq!(items[1].created_at, 6);
        post_api2
            .fetch_btm(&app.client, 2, size, 8, "one", "hey2")
            .await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[2].created_at, 5);

        app.state.set_time(9).await;
        let post_api2 = GalleryApi::new(scroll_corerction.clone());
        post_api2
            .fetch_btm(&app.client, 3, size, 9, "one two", "hey2")
            .await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 2);

        //
    }
}
