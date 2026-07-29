use catsquad_log::prelude::*;
use catsquad_shared::{LINK_WEB_INDEX, LINK_WEB_LOGIN};
use leptos::{html, prelude::*};
use leptos_router::hooks::use_navigate;
use web_sys::SubmitEvent;

use crate::{PageState, hook::Spawner, page::create_client};

#[component]
pub fn Nav() -> impl IntoView {
    let page_state = PageState::get();
    let search_input = NodeRef::<html::Div>::new();
    let navigate = use_navigate();

    // let on_login = move |_| {
    //     // let

    //     //
    // };
    let on_logout = move |_| {

        //
    };

    view! {
        <nav class="text-gray-200 flex gap-2 px-4 h-[3rem] items-center justify-between">
            <a id="banner" href=LINK_WEB_INDEX class="font-lucky font-black text-[1.3rem]">
                "CatSquad"
            </a>
            <div contenteditable=true
                 id="search"
                 node_ref=search_input
                 //on:keydown=on_enter
                 class={move || format!("w-full rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base01")}>
                 //{move || get_query_tags.get()}
            </div>
            <div class=move||format!("{}", if page_state.acc_pending() { "" } else { "hidden" })>
                <p>"loading..."</p>
            </div>
            <div class=move||format!("{}", if page_state.is_logged_in().unwrap_or_default() || page_state.acc_pending() { "hidden" } else { "" })>
                <a href=LINK_WEB_LOGIN>"Login"</a>
            </div>
            <div class=move||format!("flex gap-2 {}", if page_state.is_logged_in().unwrap_or_default() { "" } else { "hidden" })>
                // <UploadForm/>
                <a href="/upload">"Upload"</a>
                // <form method="POST" action="" on:submit=on_upload >
                //     <input type="submit" value="Upload" class="transition-all duration-300 ease-in hover:font-bold"/>
                // </form>
                <a href=move|| "/user">{move || page_state.acc_username() }</a>
                <a href=move|| "/settings">"Settings"</a>
                <form method="POST" action="" on:submit=on_logout >
                    <input type="submit" value="logout" class="transition-all duration-300 ease-in hover:font-bold"/>
                </form>
            </div>
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
