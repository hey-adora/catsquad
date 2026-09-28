use super::author_state::{AuthorStage, AuthorState};
use crate::{
    BtnSecondary, BtnSize, SVGBell,
    hook::Spawner,
    page::{create_client, index::api_gallery::GalleryApi},
};
use catsquad_web_utils::time::{micro_to_str, time_now_micro};
use leptos::prelude::*;

#[component]
pub fn ProfileAuthor(#[prop(into)] user_username: Signal<String>) -> impl IntoView {
    let spawner = Spawner::new();

    let author_state = AuthorState::new(create_client());
    let author_username = author_state.username;
    let created_at = author_state.created_at;

    Effect::new(move || {
        let username = user_username.get();
        let author_state = author_state.clone();

        spawner.spawn(async move {
            author_state.init(username).await;
        });
    });

    let author_username = move || author_username.get();
    let joined = move || micro_to_str(time_now_micro().saturating_sub(created_at.get()));

    view! {
        <div class=" sm:hidden grid grid-rows-[auto_auto] gap-2 max-w-full overflow-hidden ">
            <div class="grid grid-cols-[auto_auto] justify-start  px-2 gap-2 mb-2 " >
                <p class="self-start text-[1rem] rounded-full h-[3.2rem] w-[3.2rem] shrink-0 bg-base05"></p>
                <div class="grid grid-rows-[auto_auto_auto] gap-1">
                    <ul class="grid grid-cols-[auto_auto_auto] ml-[1rem] whitespace-nowrap gap-[1.5rem] text-[0.8rem] list-disc">
                        <li>"followers 9999"</li>
                        <li>"paws 450"</li>
                        <li>"joined "{joined}" ago"</li>
                    </ul>
                    <div class="grid grid-cols-[auto_auto] max-w-[calc(100%-3.5rem)] justify-start items-center gap-[1.5rem] ">
                        <p class="text-ellipsis overflow-hidden shrink-0 text-[1.2rem] leading-[1.2rem] font-medium text-base0F">
                            {author_username}
                        </p>
                        <BtnSecondary size=BtnSize::Small class="grid grid-cols-[auto_auto] gap-2 place-items-center ">"Follow"<SVGBell class="size-4"/></BtnSecondary>
                    </div>
                </div>
            </div>
            <ul class="pl-2 mb-2 justify-start grid grid-cols-[auto_auto_auto_auto] list-none gap-2">
                <li>"Gallery"</li>
                <li>"Favorites"</li>
                <li>"Support"</li>
                <li>"About Me"</li>
            </ul>
        </div>

        <div class="hidden sm:grid sm:col-start-2 sm:row-start-1">
            <div class="justify-start grid grid-cols-[auto_auto] ">
                <p class="self-start text-[1rem] rounded-full h-[3.2rem] w-[3.2rem] shrink-0 bg-base05"></p>
                { author_username }
            </div>
        </div>
    }
}
