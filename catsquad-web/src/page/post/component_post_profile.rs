use super::{FollowBtn, LikeBtn};
use crate::{BtnSize, Pfp, hook::Spawner, page::post::post_api::PostApi};
use catsquad_shared::UserState;
use leptos::prelude::*;

#[component]
pub fn PostProfile(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let author_username = move || post_api.author_username.get();
    let author_link = move || post_api.author_link.get();
    let pfp_hash = move || post_api.author_pfp_hash.get();
    let pfp_proccesed = move || post_api.author_pfp_proccesed.get();
    let followers = move || post_api.followers.get();
    let liked = post_api.liked;
    let followed = post_api.followed;

    let when_user_suspended = move || post_api.author_state.get().is_inactive();

    view! {
        <div class="grid gap-2 justify-start grid-cols-[auto_auto_1fr] grid-rows-[auto]">
            <Pfp class="h-[3rem] w-[3rem] shrink-0" username=author_username.clone() proccesed=pfp_proccesed hash=pfp_hash />
            <div class="grid justify-start place-content-start grid-rows-[auto_auto] grid-cols-[auto]">
                <a class="overflow-hidden whitespace-nowrap text-ellipsis max-w-full" href=author_link >
                    <span class="text-base03">"by "</span>
                    <span class="text-base0B font-bold">{ author_username }</span>
                    <Show when=when_user_suspended>
                        <span class="ml-[0.5rem] rounded-md px-[0.30rem] py-[0.15rem] bg-base0A text-base01 text-[0.8rem] font-black">"suspended"</span>
                    </Show>
                </a>
                <p class="overflow-hidden whitespace-nowrap text-ellipsis max-w-full">{followers}" followers"</p>
            </div>
            <div class="grid gap-2 justify-end grid-rows-[auto_auto] grid-cols-[auto]">
                <LikeBtn size=BtnSize::Normal class="self-end" spawner liked post_user_username=author_username post_id=post_id />
                <FollowBtn size=BtnSize::Normal class="" spawner followed post_user_username=author_username/>
            </div>
        </div>
    }
}
