use crate::{hook::Spawner, page::post::post_api::PostApi};

use super::Favorite;
use leptos::prelude::*;

#[component]
pub fn PostProfile(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let post_user_username = move || post_api.author_username.get();

    view! {
        <div class="flex justify-between place-items-start">
            <div class="flex gap-2">
                <p class="text-[1rem] rounded-full h-[3rem] w-[3rem] bg-base05"></p>
                <div class="flex flex-col gap-1">
                    <div class="flex gap-1">
                        <p class="text-[1rem] text-base03">"by"</p>
                        <a href=move || post_api.author_link.get() class="text-[1rem] font-bold text-base0B">{ move || post_api.author_username.get() }</a>
                    </div>
                    <p class="text-[1rem]">"9999 followers"</p>
                </div>
            </div>
            <Favorite post_user_username=post_user_username post_id=post_id />
        </div>
    }
}
