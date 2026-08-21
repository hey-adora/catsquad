use catsquad_log::prelude::*;
use catsquad_shared::{LINK_WEB_INDEX, LINK_WEB_LOGIN, link_relative_index_search};
use leptos::{html, prelude::*};
use leptos_router::hooks::{query_signal, use_navigate};
use web_sys::{HtmlDivElement, KeyboardEvent, SubmitEvent};

use crate::{PageState, hook::Spawner, page::create_client};

#[component]
pub fn Nav() -> impl IntoView {
    let page_state = PageState::get();
    let search_input = NodeRef::<html::Div>::new();
    let navigate = use_navigate();
    let (get_query_tags, set_query_tags) = query_signal::<String>("tags");
    let spawner = Spawner::new();

    // let on_login = move |_| {
    //     // let

    //     //
    // };
    let on_logout = move |e: SubmitEvent| {
        e.prevent_default();
        //
        spawner.spawn(async move {
            page_state.logout().await;
        });

        //
    };

    let on_enter = move |e: KeyboardEvent| {
        let key = e.key();
        trace!("key pressed {key}");
        if key.to_lowercase() != "enter" {
            return;
        }
        e.prevent_default();

        let search_text = search_input
            .get_untracked()
            .and_then(|v: HtmlDivElement| v.text_content())
            .unwrap_or_default();

        if search_text.is_empty() {
            navigate(LINK_WEB_INDEX, Default::default());
            // None
        } else {
            navigate(&link_relative_index_search(search_text), Default::default());
            // Some(search_text)
        }
    };

    Effect::new(move || {
        let (Some(search_elm), val): (Option<HtmlDivElement>, Option<String>) =
            (search_input.get(), get_query_tags.get())
        else {
            return;
        };
        if let Some(v) = val {
            search_elm.set_text_content(Some(&v));
        } else {
            search_elm.set_text_content(None);
        }
        // let val = ;
    });

    let is_loading = move || page_state.acc_pending() || spawner.is_busy.get();
    let is_logged_in = move || page_state.is_logged_in().unwrap_or_default();

    let when_guest = move || !is_logged_in() && !is_loading();
    let when_user = move || is_logged_in() && !is_loading();

    view! {
        <nav class="text-gray-200 flex gap-2 px-4 h-[3rem] items-center justify-between">
            <a id="banner" href=LINK_WEB_INDEX class="font-lucky font-black text-[1.3rem]">
                "CatSquad"
            </a>
            <div contenteditable=true
                 id="search"
                 node_ref=search_input
                 on:keydown=on_enter
                 class={move || format!("w-full rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base01")}>
                 {move || get_query_tags.get()}
            </div>
            <Show when=is_loading>
                <p>"loading..."</p>
            </Show>
            <Show when=when_guest>
                <div class="flex gap-2">
                    <a id="login_link" href=LINK_WEB_LOGIN>"Login"</a>
                </div>
            </Show>
            <Show when=when_user>
                <div class="flex gap-2">
                    <a href="/upload">"Upload"</a>
                    <a href=move|| "/user">{move || page_state.acc_username() }</a>
                    <a href=move|| "/settings">"Settings"</a>
                    <form method="POST" action="" on:submit=on_logout >
                        <input id="logout_btn" type="submit" value="logout" class="transition-all duration-300 ease-in hover:font-bold"/>
                    </form>
                </div>
            </Show>
        </nav>
    }
}

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
