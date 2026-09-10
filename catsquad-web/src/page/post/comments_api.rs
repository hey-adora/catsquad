use std::fmt::Debug;

use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{CommentRes, Order, TimeRange};
use catsquad_web_utils::prelude::*;
use leptos::{
    html::{ElementType, Textarea},
    prelude::*,
};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlTextAreaElement, MutationObserver, MutationRecord};

#[derive(Default, Clone, strum::Display, strum::EnumIs)]
pub enum CommentKind {
    #[default]
    Root,
    Reply {
        parent_id: i64,
        parent_items: RwSignal<Vec<CommentRes>, LocalStorage>,
        parent_replies_count: RwSignal<u32, LocalStorage>,
        comment: CommentRes,
    },
    Flat {
        parent_id: i64,
        parent_items: RwSignal<Vec<CommentRes>, LocalStorage>,
        parent_replies_count: RwSignal<u32, LocalStorage>,
        comment: CommentRes,
    },
    None {
        parent_id: i64,
        parent_items: RwSignal<Vec<CommentRes>, LocalStorage>,
        parent_replies_count: RwSignal<u32, LocalStorage>,
        comment: CommentRes,
    },
}

#[derive(Clone, Copy)]
pub struct CommentsApi {
    // ui
    pub items: RwSignal<Vec<CommentRes>, LocalStorage>,
    pub finished: RwSignal<bool, LocalStorage>,
    pub replies_count: RwSignal<u32, LocalStorage>,
    pub text: RwSignal<String, LocalStorage>,
    pub show_editor: RwSignal<bool, LocalStorage>,
    pub edit_mode: RwSignal<bool, LocalStorage>,
    pub err_post: RwSignal<String>,
    pub err_fetch: RwSignal<String, LocalStorage>,
    pub err_delete: RwSignal<String, LocalStorage>,
    pub err_update: RwSignal<String, LocalStorage>,

    // params
    pub post_key: StoredValue<i64, LocalStorage>,
    pub kind: StoredValue<CommentKind, LocalStorage>,
    pub fetch_count: u32,
}

impl CommentsApi {
    pub fn new(fetch_count: u32, kind: CommentKind) -> Self {
        let (replies_count, text) = match &kind {
            CommentKind::Root => (0, String::new()),
            CommentKind::Flat { comment, .. }
            | CommentKind::None { comment, .. }
            | CommentKind::Reply { comment, .. } => (comment.replies_count, comment.text.clone()),
        };

        // let has_reply_bubble = kind.is_none() && com;
        Self {
            // ui
            items: RwSignal::new_local(Vec::new()),
            finished: RwSignal::new_local(false),
            replies_count: RwSignal::new_local(replies_count),
            text: RwSignal::new_local(text),
            // has_reply_bubble,
            // is_last: RwSignal::new_local(false),
            show_editor: RwSignal::new_local(false),
            edit_mode: RwSignal::new_local(false),
            err_post: RwSignal::new(String::new()),
            err_fetch: RwSignal::new_local(String::new()),
            err_delete: RwSignal::new_local(String::new()),
            err_update: RwSignal::new_local(String::new()),
            // params
            post_key: StoredValue::new_local(0),
            kind: StoredValue::new_local(kind),
            fetch_count,
        }
    }

    fn handle_fetch_result<TError: ToString>(
        &self,
        result: Result<Vec<CommentRes>, TError>,
    ) -> Vec<CommentRes> {
        match result {
            Ok(comments) => {
                let fetch_count = self.fetch_count;
                let finished = self.finished;
                let len = comments.len() as u32;
                let is_finished = finished.get_untracked();

                if len == fetch_count && is_finished {
                    finished.set(false);
                } else if !is_finished && len < fetch_count {
                    finished.set(true);
                }

                return comments;
            }
            Err(err) => {
                let err = format!("post comments basic: {}", err.to_string());
                error!(err);
                self.err_fetch.set(err);
            }
        };
        Vec::new()
    }

    async fn fetch_replies<TSender>(
        &self,
        time: u64,
        client: &Client<TSender>,
        comment_id: i64,
        flatten: bool,
    ) -> Vec<CommentRes>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_key = self.post_key.get_value();
        if post_key == 0 {
            warn!("post key not found");
            return Vec::new();
        }
        let fetch_count = self.fetch_count;
        let finished = self.finished;
        let last_item = self.items.with_untracked(|v| v.last().cloned());
        let order = Order::OneTwoThree;
        let (time, range) = if let Some(last_item) = last_item {
            (last_item.created_at, TimeRange::More)
        } else {
            // let time = time_now_ns();
            (time, TimeRange::LessOrEqual)
        };

        let result = client
            .comment_search(
                post_key,
                comment_id,
                time,
                fetch_count as usize,
                range,
                order,
                flatten,
            )
            .send()
            .await
            .into_json()
            .await;

        self.handle_fetch_result(result)
    }

    async fn fetch_comments<TSender>(&self, time: u64, client: &Client<TSender>) -> Vec<CommentRes>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_key = self.post_key.get_value();
        if post_key == 0 {
            warn!("post key not found");
            return Vec::new();
        }
        let fetch_count = self.fetch_count;
        let finished = self.finished;
        let last_item = self.items.with_untracked(|v| v.last().cloned());
        let order = Order::ThreeTwoOne;
        let (time, range) = if let Some(last_item) = last_item {
            (last_item.created_at, TimeRange::Less)
        } else {
            // let time = time_now_ns();
            (time, TimeRange::LessOrEqual)
        };

        let result = client
            .comment_search(post_key, 0, time, fetch_count as usize, range, order, false)
            .send()
            .await
            .into_json()
            .await;
        // .get_post_comment(post_key, None, fetch_count, time_range, order, false)
        // .send_native()
        // .await;

        self.handle_fetch_result(result)
    }

    pub async fn fetch<TSender>(self, time: u64, client: &Client<TSender>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        self.err_fetch.update(|v| v.clear());
        let kind = self.kind.get_value();
        match kind {
            CommentKind::Root => {
                let comments = self.fetch_comments(time, client).await;
                if comments.is_empty() {
                    return;
                }

                self.items.update(|v| {
                    trace!("comments manual before {v:#?}");
                    v.extend(comments);
                    trace!("comments manual after {v:#?}");
                });
            }
            CommentKind::Reply { comment, .. } => {
                let comments = self
                    .fetch_replies(time, client, comment.id.clone(), false)
                    .await;
                if comments.is_empty() {
                    return;
                }

                let replies_count = self.replies_count;
                self.items.update(|v| {
                    trace!("comments manual before {v:#?}");
                    v.extend(comments);
                    trace!("comments manual after {v:#?}");

                    let len = v.len() as u32;
                    trace!("replies count {} {}", replies_count.get_untracked(), len);
                    if replies_count.get_untracked() < len {
                        replies_count.set(len);
                    }
                });
            }
            CommentKind::Flat { comment, .. } => {
                let comments = self
                    .fetch_replies(time, client, comment.id.clone(), true)
                    .await;
                if comments.is_empty() {
                    return;
                }

                let replies_count = self.replies_count;

                self.items.update(|v| {
                    trace!("comments manual before {v:#?}");
                    v.extend(comments);
                    trace!("comments manual after {v:#?}");

                    let len = v.len() as u32;
                    trace!("replies count {} {}", replies_count.get_untracked(), len);
                    if replies_count.get_untracked() < len {
                        replies_count.set(len);
                    }
                });
            }
            CommentKind::None { comment, .. } => {
                warn!("not implemented");
                //
            }
        }
    }

    fn handle_post_result<TError: ToString>(
        &self,
        result: Result<CommentRes, TError>,
    ) -> Option<CommentRes> {
        match result {
            Ok(comment) => {
                self.show_editor.set(false);
                return Some(comment);
            }
            Err(err) => {
                let err = format!("post comments basic: {}", err.to_string());
                error!(err);
                self.err_post.set(err);
            }
        };
        None
    }

    pub async fn update_comment<TSender>(self, client: &Client<TSender>, text: impl Into<String>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let text = text.into();

        let comment_key = match self.kind.get_value() {
            CommentKind::Root => {
                return;
            }

            CommentKind::Flat {
                parent_id: parent_key,
                parent_items,
                parent_replies_count,
                comment,
            }
            | CommentKind::None {
                parent_id: parent_key,
                parent_items,
                parent_replies_count,
                comment,
            }
            | CommentKind::Reply {
                parent_id: parent_key,
                parent_items,
                parent_replies_count,
                comment,
            } => comment.id,
        };

        let result = client
            .comment_update_text(comment_key, text.clone())
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => {
                self.edit_mode.set(false);
                self.text.set(text);
            }
            Err(err) => {
                let err = format!("update comment {err}");
                error!(err);
                self.err_update.set(err);
            }
        };
    }

    async fn post_comment<TSender>(
        &self,
        client: &Client<TSender>,
        text: impl Into<String>,
    ) -> Option<CommentRes>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_id = self.post_key.get_value();

        if post_id == 0 {
            error!("trying to post reply without setting post key");
            return None;
        }

        let result = client
            .comment_add(post_id, 0, text)
            .send()
            .await
            .into_json()
            .await;

        self.handle_post_result(result)
    }

    async fn post_reply<TSender>(
        &self,
        client: &Client<TSender>,
        text: impl Into<String>,
        comment_id: i64,
    ) -> Option<CommentRes>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_id = self.post_key.get_value();

        if post_id == 0 {
            error!("trying to post reply without setting post key");
            return None;
        }

        let result = client
            .comment_add(post_id, comment_id.into(), text)
            .send()
            .await
            .into_json()
            .await;

        self.handle_post_result(result)
    }

    pub async fn delete<TSender>(self, client: &Client<TSender>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        self.err_delete.update(|v| v.clear());
        match self.kind.get_value() {
            CommentKind::Root => {
                return;
            }
            CommentKind::Flat {
                parent_items: parent,
                comment,
                parent_replies_count,
                ..
            }
            | CommentKind::Reply {
                parent_items: parent,
                comment,
                parent_replies_count,
                ..
            }
            | CommentKind::None {
                parent_items: parent,
                comment,
                parent_replies_count,
                ..
            } => {
                let result = client
                    .comment_remove(comment.id.clone())
                    .send()
                    .await
                    .into_json()
                    .await;

                match result {
                    Ok(_) => {
                        // if let Some(parent) = parent {
                        let len_before = parent.with_untracked(|v| v.len());
                        parent.update(|v| {
                            *v = v
                                .clone()
                                .into_iter()
                                .filter(|v| {
                                    !(v.id == comment.id
                                        || v.parent_id.iter().any(|v| *v == comment.id))
                                })
                                .collect::<Vec<CommentRes>>();
                            // let Some(pos) = v.iter().position(|v| v.key == comment.key) else {
                            //     return;
                            // };
                            // v.remove(pos);
                        });
                        let len_after = parent.with_untracked(|v| v.len());
                        let removed = len_before.saturating_sub(len_after) as u32;

                        // let is_not_none = self.kind.with_value(|v| !v.is_none());
                        // if is_not_none {
                        // }
                        parent_replies_count.update(|v: &mut u32| {
                            *v = v.saturating_sub(removed);
                        });
                    }
                    Err(err) => {
                        let err = format!("post comments basic: {err}");
                        error!(err);
                        self.err_delete.set(err);
                    }
                };
            }
        }
    }

    pub async fn post<TSender>(self, client: &Client<TSender>, text: impl Into<String>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        // let post_key = self.post_key.get_value();
        self.err_post.update(|v| v.clear());
        let kind = self.kind.get_value();

        match kind {
            CommentKind::Root => {
                let Some(comment) = self.post_comment(client, text).await else {
                    error!("failed to post");
                    return;
                };
                self.items.update(|v| {
                    if v.is_empty() {
                        v.push(comment);
                        return;
                    }
                    v.insert(0, comment);
                });
                // self.replies_count.update(|v| *v += 1);
            }
            // CommentKind2::Comment { parent, comment }
            CommentKind::Reply {
                parent_items: parent,
                comment,
                ..
            }
            | CommentKind::Flat {
                parent_items: parent,
                comment,
                ..
            } => {
                let Some(comment) = self.post_reply(client, text, comment.id).await else {
                    error!("failed to post");
                    return;
                };

                self.items.update(|v| {
                    v.push(comment);
                });
                self.replies_count.update(|v| *v += 1);
            }
            CommentKind::None {
                parent_items: parent,
                comment,
                parent_replies_count,
                ..
            } => {
                let Some(comment) = self.post_reply(client, text, comment.id).await else {
                    error!("failed to post");
                    return;
                };

                parent.update(|v| {
                    v.push(comment);
                });
                parent_replies_count.update(|v| *v += 1);
            }
        }
    }

    pub fn is_last(&self) -> bool {
        let kind = self.kind.get_value();
        match kind {
            CommentKind::Root => false,
            // CommentKind2::Comment { parent, comment }
            CommentKind::Reply {
                parent_items: parent,
                comment,
                ..
            }
            | CommentKind::Flat {
                parent_items: parent,
                comment,
                ..
            }
            | CommentKind::None {
                parent_items: parent,
                comment,
                ..
            } => parent
                .with(|v| v.last().map(|v| v.id.clone()))
                .map(|v| v == comment.id)
                .unwrap_or_default(),
        }
    }

    pub fn init(&self, post_id: i64) {
        self.post_key.set_value(post_id);
    }
}

#[cfg(test)]
pub mod tests {
    use catsquad_api::{TestServer, auth::create_auth_cookie_str};
    // use crate::{
    //     api::{shared::post_comment::UserPostComment, tests::ApiTestApp},
    //     init_owner,
    //     view::{
    //         app::hook::api_post_comments::{CommentKind, CommentKind2, CommentsApi, CommentsApi2},
    //         logger,
    //         toolbox::prelude::*,
    //     },
    // };
    use catsquad_log::prelude::*;
    use catsquad_shared::{PostRes, PostState, uuid_to_str};
    use http::header;
    use leptos::prelude::*;
    use std::sync::Arc;

    use crate::{
        init_owner,
        page::post::comments_api::{CommentKind, CommentsApi},
    };

    async fn test_setup(db: &str) -> (Owner, TestServer, PostRes) {
        init_log();
        let owner = init_owner();

        // let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let mut app = TestServer::new(0, db).await;

        let (user1, session_token) = app
            .user_add_full("hey", "hey@heyadora.com", "pas$wAord123456789")
            .await;

        app.inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session_token)),
        )
        .await;

        let post = app
            .client
            .post_add("title1", "cat", "one")
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

        (owner, app, post)
    }

    #[tokio::test]
    pub async fn hook_comments_api_post() {
        // println!("hello");
        // init_log();
        // let owner = init_owner();

        // let server = catsquad_api::TestServer::new().await;
        // // let mut app = ApiTestApp::new(10).await;

        let mut time = 0_u64;
        // let mut t = move || {
        //     time += 1;
        //     time
        // };

        // let (user1, session_key1) = server
        //     .user_add_full("hey", "hey@heyadora.com", "pas$word123456789")
        //     .await;

        // server
        //     .inject_header(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        //     .await;

        // let post = server
        //     .client
        //     .post_add("title1", "cat", "one")
        //     .send()
        //     .await
        //     .into_res()
        //     .await
        //     .unwrap();
        let (_owner, server, post) = test_setup("hook_comments_api_post").await;

        let hook_root = CommentsApi::new(2, CommentKind::Root);
        hook_root.init(post.id);
        // assert!(!hook_root.has_reply_bubble);

        time += 1;
        server.state.set_time(time);

        hook_root.post(&server.client, "c0").await;

        time += 1;
        server.state.set_time(time);

        hook_root.post(&server.client, "c1").await;

        time += 1;
        server.state.set_time(time);

        hook_root.post(&server.client, "c2").await;

        time += 1;
        server.state.set_time(time);

        hook_root.post(&server.client, "c3").await;

        let c0 = hook_root.items.with_untracked(|v| v[3].clone());
        let hook_reply = CommentsApi::new(
            2,
            CommentKind::Reply {
                parent_id: 0,
                parent_items: hook_root.items,
                parent_replies_count: hook_root.replies_count,
                comment: c0.clone(),
            },
        );
        hook_reply.init(post.id);
        // assert!(!hook_reply.has_reply_bubble);

        time += 1;
        server.state.set_time(time);
        hook_reply.post(&server.client, "c0_r0x1").await;

        time += 1;
        server.state.set_time(time);
        hook_reply.post(&server.client, "c0_r1x1").await;

        time += 1;
        server.state.set_time(time);
        hook_reply.post(&server.client, "c0_r2x1").await;

        time += 1;
        server.state.set_time(time);
        hook_reply.post(&server.client, "c0_r3x1").await;

        let c0_r0x1 = hook_reply.items.with_untracked(|v| v[0].clone());
        let hook_flat = CommentsApi::new(
            2,
            CommentKind::Flat {
                parent_id: c0.id.clone(),
                parent_items: hook_reply.items,
                parent_replies_count: hook_reply.replies_count,
                comment: c0_r0x1.clone(),
            },
        );
        hook_flat.init(post.id);
        // assert!(!hook_flat.has_reply_bubble);

        time += 1;
        server.state.set_time(time);
        hook_flat.post(&server.client, "c0_r0x2").await;

        time += 1;
        server.state.set_time(time);
        hook_flat.post(&server.client, "c0_r1x2").await;

        time += 1;
        server.state.set_time(time);
        hook_flat.post(&server.client, "c0_r2x2").await;

        time += 1;
        server.state.set_time(time);
        hook_flat.post(&server.client, "c0_r3x2").await;

        let c0_r0x2 = hook_flat.items.with_untracked(|v| v[3].clone());
        let hook_none = CommentsApi::new(
            2,
            CommentKind::None {
                parent_id: c0_r0x1.id.clone(),
                parent_items: hook_flat.items,
                parent_replies_count: hook_flat.replies_count,
                comment: c0_r0x2.clone(),
            },
        );
        hook_none.init(post.id);
        // assert!(!hook_none.has_reply_bubble);

        time += 1;
        server.state.set_time(time);
        hook_none.post(&server.client, "c0_r0x3").await;

        time += 1;
        server.state.set_time(time);
        hook_none.post(&server.client, "c0_r1x3").await;

        time += 1;
        server.state.set_time(time);
        hook_none.post(&server.client, "c0_r2x3").await;

        time += 1;
        server.state.set_time(time);
        hook_none.post(&server.client, "c0_r3x3").await;

        let items_root = hook_root.items.get_untracked();

        assert_eq!(items_root.len(), 4);
        assert_eq!(items_root[0].text, "c3");
        assert_eq!(items_root[3].text, "c0");

        let items_reply = hook_reply.items.get_untracked();

        assert_eq!(items_reply.len(), 4);
        assert_eq!(items_reply[0].text, "c0_r0x1");
        assert_eq!(items_reply[3].text, "c0_r3x1");

        let items_flat = hook_flat.items.get_untracked();

        assert_eq!(items_flat.len(), 8);
        assert_eq!(items_flat[0].text, "c0_r0x2");
        assert_eq!(items_flat[7].text, "c0_r3x3");

        let items_none = hook_none.items.get_untracked();

        assert_eq!(items_none.len(), 0);

        // get

        let all_comments = server.state.db.comment_get_all().await.unwrap();
        let mut output = String::new();
        for comment in all_comments {
            let line = format!(
                "{} - {} - {} - {:?}\n",
                comment.id, comment.text, comment.created_at, comment.parents
            );
            output.push_str(&line);
        }
        trace!("all comments \n{output}");
        // trace!("all comments {all_comments:#?}");
        // panic!("wtf");

        let hook_root = CommentsApi::new(4, CommentKind::Root);
        hook_root.init(post.id);
        hook_root.fetch(time, &server.client).await;
        let items_root = hook_root.items.get_untracked();

        assert_eq!(items_root.len(), 4);
        assert_eq!(items_root[0].text, "c3");
        assert_eq!(items_root[3].text, "c0");

        let c0 = items_root[3].clone();
        let hook_reply = CommentsApi::new(
            4,
            CommentKind::Reply {
                parent_id: 0,
                parent_items: hook_root.items,
                parent_replies_count: hook_root.replies_count,
                comment: c0.clone(),
            },
        );
        hook_reply.init(post.id);
        hook_reply.fetch(time, &server.client).await;
        let items_reply = hook_reply.items.get_untracked();

        assert_eq!(items_reply.len(), 4);
        assert_eq!(items_reply[0].text, "c0_r0x1");
        assert_eq!(items_reply[3].text, "c0_r3x1");

        let c0_r0x1 = items_reply[0].clone();
        let hook_flat = CommentsApi::new(
            4,
            CommentKind::Flat {
                parent_id: c0.id.clone(),
                parent_items: hook_reply.items,
                parent_replies_count: hook_reply.replies_count,
                comment: c0_r0x1.clone(),
            },
        );
        hook_flat.init(post.id);
        hook_flat.fetch(time, &server.client).await;
        let items_flat = hook_flat.items.get_untracked();

        assert_eq!(items_flat.len(), 4);
        assert_eq!(items_flat[0].text, "c0_r0x2");
        assert_eq!(items_flat[3].text, "c0_r3x2");

        hook_flat.fetch(time, &server.client).await;
        let items_flat = hook_flat.items.get_untracked();

        assert_eq!(items_flat.len(), 8);
        assert_eq!(items_flat[0].text, "c0_r0x2");
        assert_eq!(items_flat[7].text, "c0_r3x3");
    }

    #[tokio::test]
    pub async fn hook_comments_api_update() {
        // println!("hello");
        // init_log();
        // let _owner = init_owner();

        // let mut app = TestServer::new().await;
        // let (user1, session_key1) = app
        //     .user_add_full("hey", "hey@heyadora.com", "pas$word123456789")
        //     .await;

        // app.inject_header(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        //     .await;

        // let post = app
        //     .client
        //     .post_add("title1", "cat", "one")
        //     .send()
        //     .await
        //     .into_res()
        //     .await
        //     .unwrap();

        let (_owner, app, post) = test_setup("hook_comments_api_update").await;

        let hook_root = CommentsApi::new(2, CommentKind::Root);
        hook_root.init(post.id);

        app.state.set_time(2);
        hook_root.post(&app.client, "c0").await;

        let c0 = hook_root.items.with_untracked(|v| v[0].clone());
        let hook_reply = CommentsApi::new(
            2,
            CommentKind::Reply {
                parent_id: 0,
                parent_items: hook_root.items,
                parent_replies_count: hook_root.replies_count,
                comment: c0.clone(),
            },
        );
        hook_reply.init(post.id);
        hook_reply.edit_mode.set(true);

        assert_eq!(hook_reply.text.get_untracked(), "c0");

        hook_reply.update_comment(&app.client, "c0_v2").await;

        assert_eq!(hook_reply.text.get_untracked(), "c0_v2");
        assert_eq!(hook_reply.edit_mode.get_untracked(), false);

        trace!("WTF");
        let hook_root = CommentsApi::new(2, CommentKind::Root);
        trace!("WTF1");
        hook_root.init(post.id);
        trace!("WTF2");
        hook_root.fetch(2, &app.client).await;
        trace!("WTF3");

        let items_root = hook_root.items.get_untracked();
        let c0 = hook_root.items.with_untracked(|v| v[0].clone());

        assert_eq!(items_root.len(), 1);
        assert_eq!(c0.text, "c0_v2");
    }

    #[tokio::test]
    pub async fn hook_comments_api_delete() {
        // println!("hello");
        // init_log();
        // let owner = init_owner();

        // let mut app = TestServer::new().await;

        let mut time = 0_u64;
        // let mut t = move || {
        //     time += 1;
        //     time
        // };

        // let (user1, session_key1) = app
        //     .user_add_full("hey", "hey@heyadora.com", "pas$word123456789")
        //     .await;

        // app.inject_header(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        //     .await;
        // // app.api.auth_token_overwrite = auth_token.clone();

        // let post = app
        //     .client
        //     .post_add("title1", "cat", "one")
        //     .send()
        //     .await
        //     .into_res()
        //     .await
        //     .unwrap();
        let (_owner, app, post) = test_setup("hook_comments_api_delete").await;

        let hook_root = CommentsApi::new(2, CommentKind::Root);
        hook_root.init(post.id);

        time += 1;
        app.state.set_time(time);
        hook_root.post(&app.client, "c0").await;

        time += 1;
        app.state.set_time(time);
        hook_root.post(&app.client, "c1").await;

        hook_root.delete(&app.client).await;
        let items_root = hook_root.items.get();
        assert_eq!(items_root.len(), 2);

        let c0 = hook_root.items.with_untracked(|v| v[1].clone());

        let hook_reply = CommentsApi::new(
            2,
            CommentKind::Reply {
                parent_id: 0,
                parent_items: hook_root.items,
                parent_replies_count: hook_root.replies_count,
                comment: c0.clone(),
            },
        );
        hook_reply.init(post.id);

        time += 1;
        app.state.set_time(time);
        hook_reply.post(&app.client, "c0_r0x1").await;

        time += 1;
        app.state.set_time(time);
        hook_reply.post(&app.client, "c0_r1x1").await;

        let items_reply = hook_reply.items.get();
        assert_eq!(items_reply.len(), 2);

        let c0_r0x1 = hook_reply.items.with_untracked(|v| v[0].clone());

        let hook_flat = CommentsApi::new(
            2,
            CommentKind::Flat {
                parent_id: c0.id.clone(),
                parent_items: hook_reply.items,
                parent_replies_count: hook_reply.replies_count,
                comment: c0_r0x1.clone(),
            },
        );
        hook_flat.init(post.id);

        time += 1;
        app.state.set_time(time);
        hook_flat.post(&app.client, "c0_r0x2").await;

        time += 1;
        app.state.set_time(time);
        hook_flat.post(&app.client, "c0_r1x2").await;

        let items_flat = hook_flat.items.get();
        assert_eq!(items_flat.len(), 2);

        let c0_r0x2 = hook_flat.items.with_untracked(|v| v[0].clone());

        let hook_none = CommentsApi::new(
            2,
            CommentKind::None {
                parent_id: c0_r0x1.id.clone(),
                parent_items: hook_flat.items,
                parent_replies_count: hook_flat.replies_count,
                comment: c0_r0x2,
            },
        );
        hook_none.init(post.id);

        time += 1;
        app.state.set_time(time);
        hook_none.post(&app.client, "c0_r0x3").await;

        time += 1;
        app.state.set_time(time);
        hook_none.post(&app.client, "c0_r1x3").await;

        let items_flat = hook_flat.items.get();
        assert_eq!(items_flat.len(), 4);

        // let c0_r0x3 = hook_flat.items.with_untracked(|v| v[3].clone());
        // let hook_none2 = CommentsApi2::new(
        //     &app.api,
        //     2,
        //     CommentKind2::None {
        //         parent_items: hook_flat.items,
        //         parent_replies_count: hook_flat.replies_count,
        //         comment: c0_r0x3,
        //     },
        // );
        // hook_none2.observe_only(post.id.clone());
        //
        // (app.set_time(t()).await, hook_none2.post("c0_r0x4").await);
        //
        // let items_flat = hook_flat.items.get();
        // assert_eq!(items_flat.len(), 5);

        {
            let replies_count = hook_flat.replies_count.get_untracked();
            assert_eq!(replies_count, 4);

            hook_none.delete(&app.client).await;

            let items_flat = hook_flat.items.get_untracked();
            let replies_count = hook_flat.replies_count.get_untracked();

            assert_eq!(replies_count, 1);
            assert_eq!(items_flat.len(), 1);
            assert_eq!(items_flat[0].text, "c0_r1x2");

            // assert_eq!(items_flat[2].text, "c0_r1x3");
        }

        {
            let replies_count = hook_reply.replies_count.get_untracked();
            assert_eq!(replies_count, 2);

            hook_flat.delete(&app.client).await;

            let items_reply = hook_reply.items.get();
            let replies_count = hook_reply.replies_count.get_untracked();

            assert_eq!(replies_count, 1);
            assert_eq!(items_reply.len(), 1);
            assert_eq!(items_reply[0].text, "c0_r1x1");
        }

        {
            let replies_count = hook_root.replies_count.get_untracked();
            assert_eq!(replies_count, 0);

            hook_reply.delete(&app.client).await;

            let items_root = hook_root.items.get();
            let replies_count = hook_root.replies_count.get_untracked();

            assert_eq!(replies_count, 0);
            assert_eq!(items_root.len(), 1);
            assert_eq!(items_root[0].text, "c1");
        }
    }

    // #[cfg(test)]
    #[tokio::test]
    pub async fn hook_comments_api_get() {
        // println!("hello");
        // init_log();
        // let owner = init_owner();

        // // let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        // let mut app = TestServer::new().await;

        // let (user1, session_key1) = app
        //     .user_add_full("hey", "hey@heyadora.com", "pas$word123456789")
        //     .await;

        // app.inject_header(header::COOKIE, create_auth_cookie_str(session_key1.clone()))
        //     .await;

        // let post = app
        //     .client
        //     .post_add("title1", "cat", "one")
        //     .send()
        //     .await
        //     .into_res()
        //     .await
        //     .unwrap();
        let (_owner, app, post) = test_setup("hook_comments_api_get").await;

        let hook_root = CommentsApi::new(2, CommentKind::Root);
        hook_root.post_key.set_value(post.id);

        // (app.set_time(0).await, hook_root.post("comment0").await);
        // let comment0 = hook_root.items.with_untracked(|v| v[0].clone());

        let mut time = 0_u64;
        // let mut get_time = move || {
        //     time += 1;
        //     time
        // };

        let fn_comment_add = async |parent: i64, text: &str| {
            // app.state.set_time(get_time()).await;
            app.client
                .comment_add(post.id, parent, text)
                .send()
                .await
                .into_json()
                .await
                .unwrap()
        };

        // let comment0 = app
        //     .client
        //     .comment_add(post.key.clone(), String::new(), "wowza".to_string())
        //     .send()
        //     .await
        //     .into_res()
        //     .await
        //     .unwrap();
        //
        time += 1;
        app.state.set_time(time);
        let comment0 = fn_comment_add(0, "comment0").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0 = fn_comment_add(comment0.id.clone(), "comment0_reply0").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0_reply0 =
            fn_comment_add(comment0_reply0.id.clone(), "comment0_reply0_reply0").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0_times_3 =
            fn_comment_add(comment0_reply0_reply0.id.clone(), "comment0_reply0_times_3").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0_times_4 = fn_comment_add(
            comment0_reply0_times_3.id.clone(),
            "comment0_reply0_times_4",
        )
        .await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0_times_5 = fn_comment_add(
            comment0_reply0_times_4.id.clone(),
            "comment0_reply0_times_5",
        )
        .await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0_times_6 = fn_comment_add(
            comment0_reply0_times_5.id.clone(),
            "comment0_reply0_times_6",
        )
        .await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0_reply1 =
            fn_comment_add(comment0_reply0.id.clone(), "comment0_reply0_reply1").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0_reply2 =
            fn_comment_add(comment0_reply0.id.clone(), "comment0_reply0_reply2").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply0_reply3 =
            fn_comment_add(comment0_reply0.id.clone(), "comment0_reply0_reply3").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply1 = fn_comment_add(comment0.id.clone(), "comment0_reply1").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply2 = fn_comment_add(comment0.id.clone(), "comment0_reply2").await;
        time += 1;
        app.state.set_time(time);
        let comment0_reply3 = fn_comment_add(comment0.id.clone(), "comment0_reply3").await;
        time += 1;
        app.state.set_time(time);
        let comment1 = fn_comment_add(0, "comment1").await;
        time += 1;
        app.state.set_time(time);
        let comment2 = fn_comment_add(0, "comment2").await;
        time += 1;
        app.state.set_time(time);
        let comment3 = fn_comment_add(0, "comment3").await;

        let replies_count = hook_root.replies_count.get_untracked();
        assert_eq!(replies_count, 0);

        hook_root.fetch(time, &app.client).await;

        let post_comments = hook_root.items.get_untracked();
        let replies_count = hook_root.replies_count.get_untracked();

        assert_eq!(replies_count, 0);
        assert_eq!(post_comments.len(), 2);
        assert_eq!(post_comments[0], comment3);
        assert_eq!(post_comments[1], comment2);

        hook_root.fetch(time, &app.client).await;

        let post_comments = hook_root.items.get_untracked();
        assert_eq!(post_comments.len(), 4);
        assert_eq!(post_comments[0], comment3);
        assert_eq!(post_comments[1], comment2);
        assert_eq!(post_comments[2], comment1);
        assert_eq!(post_comments[3].id, comment0.id);

        let comment4 = fn_comment_add(0, "comment4").await;
        // let comment4 = app
        //     .add_post_comment(
        //         4,
        //         &auth_token,
        //         post.key.clone(),
        //         None,
        //         "comment4".to_string(),
        //     )
        //     .await
        //     .unwrap();

        hook_root.fetch(time, &app.client).await;

        let post_comments = hook_root.items.get_untracked();
        assert_eq!(post_comments.len(), 4);
        assert_eq!(post_comments[0], comment3);
        assert_eq!(post_comments[1], comment2);
        assert_eq!(post_comments[2], comment1);
        assert_eq!(post_comments[3].id, comment0.id);

        let hook_comment = CommentsApi::new(
            2,
            CommentKind::Reply {
                parent_id: 0,
                parent_items: hook_root.items,
                parent_replies_count: hook_root.replies_count,
                comment: comment0.clone(),
            },
        );
        hook_comment.post_key.set_value(post.id);

        hook_comment.fetch(time, &app.client).await;
        let comment0_replies = hook_comment.items.get_untracked();

        assert_eq!(comment0_replies.len(), 2);
        assert_eq!(comment0_replies[0].id, comment0_reply0.id);
        assert_eq!(comment0_replies[0].replies_count, 4);
        assert_eq!(comment0_replies[1], comment0_reply1);

        hook_comment.fetch(time, &app.client).await;
        let comment0_replies = hook_comment.items.get_untracked();

        assert_eq!(comment0_replies.len(), 4);
        assert_eq!(comment0_replies[0].id, comment0_reply0.id);
        assert_eq!(comment0_replies[0].replies_count, 4);
        assert_eq!(comment0_replies[1], comment0_reply1);
        assert_eq!(comment0_replies[2], comment0_reply2);
        assert_eq!(comment0_replies[3], comment0_reply3);

        let hook_reply = CommentsApi::new(
            2,
            CommentKind::Reply {
                parent_id: comment0.id.clone(),
                parent_items: hook_comment.items,
                parent_replies_count: hook_comment.replies_count,
                comment: comment0_reply0.clone(),
            },
        );
        hook_reply.post_key.set_value(post.id);

        // trace!("yo yo yo yo did u run or no");
        hook_reply.fetch(time, &app.client).await;
        let comment0_reply0_replies = hook_reply.items.get_untracked();
        // trace!("WHAT THE F*CK: {comment0_reply0_replies:#?}");
        // hook_reply.items.update(|v| {
        //     trace!("WHAT THE F*CK 2: {v:#?}");
        // });

        assert_eq!(comment0_reply0_replies.len(), 2);
        assert_eq!(comment0_reply0_replies[0].id, comment0_reply0_reply0.id);
        assert_eq!(comment0_reply0_replies[0].replies_count, 1);
        assert_eq!(comment0_reply0_replies[1], comment0_reply0_reply1);

        hook_reply.fetch(time, &app.client).await;
        let comment0_reply0_replies = hook_reply.items.get_untracked();

        assert_eq!(comment0_reply0_replies.len(), 4);
        assert_eq!(comment0_reply0_replies[0].id, comment0_reply0_reply0.id);
        assert_eq!(comment0_reply0_replies[0].replies_count, 1);
        assert_eq!(comment0_reply0_replies[1], comment0_reply0_reply1);
        assert_eq!(comment0_reply0_replies[2], comment0_reply0_reply2);
        assert_eq!(comment0_reply0_replies[3], comment0_reply0_reply3);

        let hook_flat = CommentsApi::new(
            2,
            CommentKind::Flat {
                parent_id: comment0_reply0.id.clone(),
                parent_items: hook_reply.items,
                parent_replies_count: hook_reply.replies_count,
                comment: comment0_reply0_reply0.clone(),
            },
        );
        hook_flat.post_key.set_value(post.id);

        trace!("comment0_reply0_reply0 {comment0_reply0_reply0:#?}");

        let replies_count = hook_flat.replies_count.get_untracked();
        assert_eq!(replies_count, 0);

        hook_flat.fetch(time, &app.client).await;

        let comment0_reply0_reply0_replies = hook_flat.items.get_untracked();
        let replies_count = hook_flat.replies_count.get_untracked();
        trace!("comment0_reply0_reply0_replies {comment0_reply0_reply0_replies:#?}");

        assert_eq!(replies_count, 2);
        assert_eq!(comment0_reply0_reply0_replies.len(), 2);
        assert_eq!(
            comment0_reply0_reply0_replies[0].id,
            comment0_reply0_times_3.id
        );
        assert_eq!(
            comment0_reply0_reply0_replies[1].id,
            comment0_reply0_times_4.id
        );

        hook_flat.fetch(time, &app.client).await;
        let comment0_reply0_reply0_replies = hook_flat.items.get_untracked();
        let replies_count = hook_flat.replies_count.get_untracked();

        assert_eq!(replies_count, 4);
        assert_eq!(comment0_reply0_reply0_replies.len(), 4);
        assert_eq!(
            comment0_reply0_reply0_replies[0].id,
            comment0_reply0_times_3.id
        );
        assert_eq!(
            comment0_reply0_reply0_replies[1].id,
            comment0_reply0_times_4.id
        );
        assert_eq!(
            comment0_reply0_reply0_replies[2].id,
            comment0_reply0_times_5.id
        );
        assert_eq!(
            comment0_reply0_reply0_replies[3].id,
            comment0_reply0_times_6.id
        );
        assert_eq!(hook_flat.finished.get_untracked(), false);

        hook_flat.fetch(time, &app.client).await;
        let comment0_reply0_reply0_replies = hook_flat.items.get_untracked();

        assert_eq!(comment0_reply0_reply0_replies.len(), 4);
        assert_eq!(hook_flat.finished.get_untracked(), true);

        // let (mut browser, mut handler) =
        // Browser::launch(BrowserConfig::builder().with_head().build().unwrap())
        //     .await
        //     .unwrap();
        // let mut rt = tokio::runtime::Builder::new_current_thread()
        //     .enable_time()
        //     .enable_io()
        //     .build()
        //     .unwrap();
        // rt.block_on(async {
        // });
    }

    // #[tokio::test]
    // pub fn comments_hook() {
    //     console_error_panic_hook::set_once();
    //     logger::simple_web_logger_init();
    //     tracing::debug!("yo wtf");
    //     debug!("hello");

    //     stringify!({
    //         let list_root = Vec::<UserPostComment>::new();
    //         let list_normal = Vec::<UserPostComment>::new();
    //         let list_reply = Vec::<UserPostComment>::new();
    //         let list_reply2 = Vec::<UserPostComment>::new();
    //         let list_flat = Vec::<UserPostComment>::new();
    //         let list_flat1 = Vec::<UserPostComm>::new();
    //     });

    //     //init_test_log();
    //     // leptos::mount::hydrate_body(App);
    //     // _ = Executor::init_wasm_bindgen();
    //     // let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
    //     //
    //     // let api = CommentsApi::new(CommentKind::Root);
    //     // api.fetch_btm();
    //     //
    //     // let list_root = Vec::<UserPostComment>::new();
    //     // let list_normal = Vec::<UserPostComment>::new();
    //     // let list_reply = Vec::<UserPostComment>::new();
    //     // let list_reply2 = Vec::<UserPostComment>::new();
    //     // let list_flat = Vec::<UserPostComment>::new();

    //     // let result = Command::new("cargo")
    //     //     .args([
    //     //         "build",
    //     //         "--package=artbounty",
    //     //         "--lib",
    //     //         "--target=wasm32-unknown-unknown",
    //     //         "--features=hydrate",
    //     //         "--profile",
    //     //         "wasm-debug",
    //     //     ])
    //     //     .output()
    //     //     .await
    //     //     .expect("failed to execute process");
    //     // debug!("{result:#?}");

    //     // let a = Command::new();

    //     // let a = RwSignal::new(0);

    //     // a.set(69);

    //     // println!("yoyyo");

    //     // trace!("hello {}", a.get_untracked());
    // }

    // TODO add error test
}
