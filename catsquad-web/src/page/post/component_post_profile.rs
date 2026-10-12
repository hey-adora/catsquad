use super::Favorite;
use crate::{Pfp, hook::Spawner, page::post::post_api::PostApi};
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

    view! {
        <div class="flex justify-between place-items-start">
            <div class="flex gap-2">
                <a href=author_link>
                    <Pfp username=author_username.clone() proccesed=pfp_proccesed hash=pfp_hash class="h-[3rem] w-[3rem] shrink-0"/>
                </a>
                <div class="flex flex-col gap-1">
                    <div class="flex gap-1">
                        <a href=author_link class="">
                            <span class="text-base03">"by"</span>
                            <span class="text-base0B font-bold">{ author_username }</span>
                        </a>
                    </div>
                    <p class="text-[1rem]">"9999 followers"</p>
                </div>
            </div>
            <Favorite post_user_username=author_username post_id=post_id />
        </div>
    }
}
