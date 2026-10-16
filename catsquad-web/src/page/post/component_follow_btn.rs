use super::post_like_state::PostLikeState;
use crate::{
    BtnSecondary, BtnSize, PageState, SVGBell, SVGStar,
    hook::{FollowState, Spawner},
    page::{create_client, post::post_like_state::LikeState},
};
use leptos::prelude::*;

#[component]
pub fn FollowBtn(
    spawner: Spawner,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] size: Signal<BtnSize>,
    followed: RwSignal<bool>,
    #[prop(into)] post_user_username: Signal<String>,
) -> impl IntoView {
    let page = PageState::get();
    let follow_state =
        FollowState::new_with_state(create_client(), post_user_username.into(), followed);
    let on_click = move |_| {
        spawner.spawn(async move {
            follow_state.follow_toggle().await;
        })
    };

    let is_visible_fn = move || -> bool {
        let post_user_username = post_user_username.get();
        let user_username = page.acc_username();
        let is_my_post = user_username == post_user_username;
        let is_logged_in = page.is_logged_in().unwrap_or_default();
        is_logged_in && !is_my_post
    };

    let class_btn = move || format!("flex gap-2 place-items-center {}", class.get());
    let class_bell = move || format!("size-4 {}", if followed.get() { "fill-base05" } else { "" });

    view! {
        <Show when=is_visible_fn>
            <BtnSecondary size class=class_btn id="btn_favorite" on_click=on_click >
                <span class="mt-[0.1rem]">"Follow"</span>
                <SVGBell class=class_bell/>
            </BtnSecondary>
        </Show>
    }
}
