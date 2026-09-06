use super::post_like_state::PostLikeState;
use crate::{
    BtnSecondary, PageState, SVGStar,
    hook::Spawner,
    page::{create_client, post::post_like_state::LikeState},
};
use leptos::prelude::*;

#[component]
pub fn Favorite(
    #[prop(optional, into)] post_id: Signal<i64>,
    #[prop(optional, into)] post_user_username: Signal<String>,
) -> impl IntoView {
    let page = PageState::get();
    let post_like = PostLikeState::new(create_client());
    let spawner = Spawner::new();

    Effect::new(move || {
        let post_id = post_id.get();
        if post_id == 0 {
            return;
        }
        spawner.spawn(async move {
            post_like.init(post_id).await;
        });
    });

    let is_visible_fn = move || -> bool {
        let post_user_username = post_user_username.get();
        let user_username = page.acc_username();
        let is_my_post = user_username == post_user_username;
        let is_logged_in = page.is_logged_in().unwrap_or_default();
        is_logged_in && !is_my_post
    };
    let is_post_loading_fn = move || post_like.state.get() == LikeState::Loading;
    let is_post_liked_fn = move || post_like.state.get() == LikeState::Liked;
    let toggle_like_fn = move || {
        spawner.spawn(async move {
            post_like.toggle_like().await;
        });
    };

    view! {
        <Show when=is_visible_fn>
            <Show when=is_post_loading_fn>
                <BtnSecondary class=move || "flex gap-2 place-items-center" id=move || "btn_favorite_loading">
                    <span class="mt-[0.1rem]">"Loading..."</span>
                </BtnSecondary>
            </Show>
            <Show when=move||!is_post_loading_fn()>
                <BtnSecondary class=move || "flex gap-2 place-items-center" id=move || "btn_favorite" on_click=move|_|toggle_like_fn()>
                    <span class="mt-[0.1rem]">"Favorite"</span>
                    <SVGStar class=move||"shrink-0 w-[1.5rem] pb-[0.1rem]" fill=move||is_post_liked_fn() />
                </BtnSecondary>
            </Show>
        </Show>
    }
}
