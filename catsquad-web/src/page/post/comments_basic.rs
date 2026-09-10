use crate::{
    hook::{InfiniteScrollFn, Spawner},
    page::{
        create_client,
        post::comments_api::{CommentKind, CommentsApi},
    },
};
use catsquad_shared::CommentRes;
use catsquad_web_utils::time::time_now_micro;
use leptos::prelude::*;
use web_sys::Element;

#[derive(Copy, Clone)]
pub struct CommentsBaisc {
    pub replies_count: RwSignal<u32, LocalStorage>,
    pub comments_manual: CommentsApi,
    pub err_post: RwSignal<String>,
    pub items: RwSignal<Vec<CommentRes>, LocalStorage>,
    pub infinite_fn: InfiniteScrollFn,
}

impl CommentsBaisc {
    pub fn new(spawner: Spawner) -> Self {
        let comments_manual = CommentsApi::new(5, CommentKind::Root);

        let infinite_fn = InfiniteScrollFn::new(move |_a| {
            spawner.spawn(async move {
                let time = time_now_micro();
                let client = create_client();
                comments_manual.fetch(time, &client).await;
            });
        });

        Self {
            comments_manual,
            err_post: comments_manual.err_post,
            replies_count: comments_manual.replies_count,
            items: comments_manual.items,
            infinite_fn,
        }
    }

    pub async fn init(self, comment_container: Element, post_id: i64) {
        let time = time_now_micro();
        let client = create_client();
        self.comments_manual.init(post_id);
        self.comments_manual.fetch(time, &client).await;
        self.infinite_fn.observe_only(comment_container);
    }
}
