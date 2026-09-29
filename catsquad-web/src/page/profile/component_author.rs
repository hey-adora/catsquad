use super::author_state::{AuthorStage, AuthorState};
use crate::{
    Btn, BtnSecondary, BtnSize, LinkSecondary, SVGArrowDown, SVGBell,
    hook::Spawner,
    page::{create_client, index::api_gallery::GalleryApi},
};
use catsquad_web_utils::time::{micro_to_str, time_now_micro};
use leptos::prelude::*;

#[component]
pub fn ProfileAuthor(#[prop(into)] user_username: Signal<String>) -> impl IntoView {
    let spawner = Spawner::new();
    let is_open = RwSignal::new(true);

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
    let toggle_open = move |_| is_open.update(|v| *v = !*v);

    let when_is_open = move || is_open.get();
    let when_is_closed = move || !when_is_open();

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

        <Show when=when_is_open>
            <div class="hidden sm:grid sm:col-start-2 sm:row-start-1 grid-rows-[auto_auto_auto_auto_1fr] gap-4 px-6 py-4 bg-base01 rounded-lg">
                <div class="flex  gap-4 ">
                    <Btn class="self-start bg-base02 " on_click=toggle_open><SVGArrowDown stroke=2.5 class="size-6 rotate-90 "/></Btn>
                    <a class="self-center text-[1.2rem] underline">"gallery"</a>
                    <a class="self-center text-[1.2rem]">"favorites"</a>
                </div>
                <div class="place-items-center grid grid-cols-[auto_auto] gap-4 ">
                    <p class="self-start text-[1rem] rounded-full size-[5rem] shrink-0 bg-base05"></p>
                    <div class="grid grid-rows-[auto_auto_auto] gap-2 place-items-start" >
                        <p class="text-[1.2rem] leading-[1.2rem] font-medium text-base0F max-w-[10rem] break-all ">{ author_username }</p>
                        <BtnSecondary size=BtnSize::Normal class="grid grid-cols-[auto_auto] gap-2 place-items-center ">"Follow"<SVGBell class="size-4"/></BtnSecondary>
                        <ul class="ml-[1rem] whitespace-nowrap gap-[1.5rem] text-[1rem] list-disc">
                            <li>"followers 9999"</li>
                            <li>"paws 450"</li>
                            <li>"joined "{joined}" ago"</li>
                        </ul>
                    </div>
                </div>
                <div class="grid grid-rows-[auto_auto] gap-2">
                    <p class="text-[1.2rem] leading-[1.2rem] font-medium text-base0F">"support"</p>
                    <p >"patreon link or something"</p>
                </div>
                <div class="grid grid-rows-[auto_auto] gap-2">
                    <p class="text-[1.2rem] leading-[1.2rem] font-medium text-base0F">"about me"</p>
                    <p >"IM AMAZING"</p>
                </div>
            </div>
        </Show>

        <Show when=when_is_closed>
            <div class="place-items-center hidden sm:grid sm:col-start-2 sm:row-start-1 grid-rows-[auto_auto_auto_auto_1fr] gap-4 px-4 py-2 bg-base01 rounded-lg">
                <Btn class="self-start bg-base02 " size=BtnSize::Small on_click=toggle_open><SVGArrowDown stroke=2.5 class="size-6 -rotate-90 "/></Btn>
                <p class="self-start text-[1rem] rounded-full size-[2.5rem] shrink-0 bg-base05"></p>
                <a class="rounded-lg bg-base02 self-center text-[1.2rem] px-4 py-2 font-bold ">"G"</a>
                <a class="rounded-lg bg-base02 self-center text-[1.2rem] px-4 py-2 font-bold ">"F"</a>
            </div>
        </Show>
    }
}

// <Btn class="self-start bg-base02 "><SVGArrowDown stroke=2.5 class="size-6 rotate-90 "/></Btn>
