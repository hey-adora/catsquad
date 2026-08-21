use catsquad_log::prelude::*;
use catsquad_shared::{LINK_WEB_INDEX, LINK_WEB_LOGIN, link_relative_index_search};
use catsquad_web_utils::prelude::rem_to_px;
use leptos::{html, prelude::*};
use leptos_router::hooks::{query_signal, use_navigate};
use wasm_bindgen::JsCast;
use web_sys::{HtmlDivElement, HtmlInputElement, HtmlTextAreaElement, KeyboardEvent, SubmitEvent};

use crate::{AutoTextArea, LinkPrimary, PageState, SVGUpload, hook::Spawner, page::create_client};

mod profile_component;
mod search_bar_component;

use profile_component::Profile;
use search_bar_component::SearchBar;

#[component]
pub fn Nav() -> impl IntoView {
    let page_state = PageState::get();
    let spawner = Spawner::new();

    let is_loading = move || page_state.acc_pending() || spawner.is_busy.get();
    let is_logged_in = move || page_state.is_logged_in().unwrap_or_default();

    let when_guest = move || !is_logged_in() && !is_loading();
    let when_user = move || is_logged_in() && !is_loading();

    view! {
        <nav class="text-gray-200 flex gap-2 px-4 h-[3rem] items-center justify-between">
            <a id="banner" href=LINK_WEB_INDEX class="font-lucky font-black text-[1.3rem]">
                "CatSquad"
            </a>
            <SearchBar/>
            <Show when=is_loading>
                <p>"loading..."</p>
            </Show>
            <Show when=when_guest>
                <div class="flex gap-2">
                    <LinkPrimary class="" id="login_link" link=LINK_WEB_LOGIN>"Login"</LinkPrimary>
                </div>
            </Show>
            <Show when=when_user>
                <div class="flex gap-2 items-center">
                    <a href="/upload" class="rounded-full bg-base0B py-2 px-2">
                        <SVGUpload stroke="0.5" class=move||"text-base01 size-4" />
                    </a>
                    <Profile spawner/>
                </div>
            </Show>
        </nav>
    }
}

// <a href=move|| "/user">{move || page_state.acc_username() }</a>
// <a href=move|| "/settings">"Settings"</a>
// contenteditable=true
// <AutoTextArea
//      id=move||"search"
//      min_height=rem_to_px(2).unwrap()
//      placeholder=move||"Search"
//      node_ref=search_input
//      on_enter=on_search
//      class=move || " mx-auto w-full rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base03">
//      {move || get_query_tags.get()}
// </AutoTextArea>
// #[component]
// pub fn UploadForm() -> impl IntoView {
//     let spawner = Spawner::new();
//     let on_upload = move |e: SubmitEvent| {
//         e.prevent_default();
//         let client = create_client();
//         spawner.spawn(async move {
//             let result = client
//                 .post_add("", "", "")
//                 .await
//                 .send()
//                 .await
//                 .into_res()
//                 .await;
//             match result {
//                 Ok(v) => {
//                     // v.k
//                 }

//                 Err(err) => {
//                     //
//                 }
//             }
//         });
//     };
//     view! {
//         <form on:submit=on_upload>

//             <input type="submit" value="Upload" class="transition-all duration-300 ease-in hover:font-bold"/>
//         </form>
//     }
// }
