use super::post_like_state::PostLikeState;
use crate::{
    BtnSecondary, PageState, SVGStar,
    hook::Spawner,
    page::{create_client, post::post_like_state::LikeState},
};
use leptos::prelude::*;

#[component]
pub fn LikeBtn(
    liked: RwSignal<bool>,
    #[prop(into)] post_id: Signal<i64>,
    #[prop(into)] post_user_username: Signal<String>,
) -> impl IntoView {
    let page = PageState::get();
    let post_like = PostLikeState::new_with_data(create_client(), post_id, liked);
    let spawner = Spawner::new();

    let is_visible_fn = move || -> bool {
        let post_user_username = post_user_username.get();
        let user_username = page.acc_username();
        let is_my_post = user_username == post_user_username;
        let is_logged_in = page.is_logged_in().unwrap_or_default();
        is_logged_in && !is_my_post
    };
    let is_post_loading_fn = move || post_id.get() == 0;
    let is_post_loaded_fn = move || !is_post_loading_fn();

    let is_post_liked_fn = move || post_like.liked.get();
    let toggle_like_fn = move |_| {
        spawner.spawn(async move {
            post_like.toggle_like().await;
        });
    };

    // TODO i think i can use Btn loading param instead of this Show bs

    view! {
        <Show when=is_visible_fn>
            <Show when=is_post_loading_fn>
                <BtnSecondary class="flex gap-2 place-items-center" id="btn_favorite_loading">
                    <span class="mt-[0.1rem]">"Loading..."</span>
                </BtnSecondary>
            </Show>
            <Show when=is_post_loaded_fn>
                <BtnSecondary class="flex gap-2 place-items-center" id="btn_favorite" on_click=toggle_like_fn>
                    <span class="mt-[0.1rem]">"Favorite"</span>
                    <SVGStar class="shrink-0 w-[1.5rem] pb-[0.1rem]" fill=move||is_post_liked_fn() />
                </BtnSecondary>
            </Show>
        </Show>
    }
}
