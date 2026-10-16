use super::post_like_state::PostLikeState;
use crate::{
    BtnSecondary, BtnSize, PageState, SVGStar,
    hook::Spawner,
    page::{create_client, post::post_like_state::LikeState},
};
use leptos::prelude::*;

#[component]
pub fn LikeBtn(
    spawner: Spawner,
    liked: RwSignal<bool>,
    #[prop(into)] post_id: Signal<i64>,
    #[prop(into)] post_user_username: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] size: Signal<BtnSize>,
) -> impl IntoView {
    let page = PageState::get();
    let post_like = PostLikeState::new_with_data(create_client(), post_id, liked);
    // let spawner = Spawner::new();

    let is_visible_fn = move || -> bool {
        let post_user_username = post_user_username.get();
        let user_username = page.acc_username();
        let is_my_post = user_username == post_user_username;
        let is_logged_in = page.is_logged_in().unwrap_or_default();
        is_logged_in && !is_my_post
    };
    let is_post_loading_fn = move || spawner.is_busy.get();

    let is_post_liked_fn = move || post_like.liked.get();
    let toggle_like_fn = move |_| {
        spawner.spawn(async move {
            post_like.toggle_like().await;
        });
    };

    let class = move || format!("flex gap-2 place-items-center {}", class.get());
    // size=BtnSize::Small
    view! {
        <Show when=is_visible_fn>
            <BtnSecondary size is_loading=is_post_loading_fn class=class id="btn_favorite" on_click=toggle_like_fn>
                <span class="mt-[0.1rem]">"Favorite"</span>
                <SVGStar class="size-4 pb-[0.1rem]" fill=move||is_post_liked_fn() />
            </BtnSecondary>
        </Show>
    }
}
