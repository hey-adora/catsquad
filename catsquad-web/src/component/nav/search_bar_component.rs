use crate::SVGSearch;
use catsquad_log::prelude::*;
use catsquad_shared::{LINK_WEB_INDEX, link_relative_index_search};
use leptos::{
    html::{self, Textarea},
    prelude::*,
};
use leptos_router::hooks::{query_signal, use_navigate};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlTextAreaElement, KeyboardEvent};

#[component]
pub fn SearchBar() -> impl IntoView {
    let search_input = NodeRef::<html::Input>::new();
    let navigate = use_navigate();
    let (get_query_tags, set_query_tags) = query_signal::<String>("tags");

    let on_search = move |e: web_sys::KeyboardEvent| {
        let key = e.key();
        trace!("key pressed {key}");
        if key.to_lowercase() != "enter" {
            return;
        }
        e.prevent_default();
        let search_text = e
            .target()
            .map(|v| (v.unchecked_into::<HtmlInputElement>()).value())
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
        trace!("nav effect 0");
        let (Some(search_elm), val): (Option<HtmlInputElement>, Option<String>) =
            (search_input.get(), get_query_tags.get())
        else {
            return;
        };
        trace!("nav effect 1");
        if let Some(v) = val {
            search_elm.set_value(&v);
        } else {
            search_elm.set_value("");
        }
        // let val = ;
    });

    // let value = move || {
    //     let value = get_query_tags.get().unwrap_or("wtf".to_string());
    //     trace!("nav value set {value}");
    //     value
    // };

    view! {
        <div class="flex gap-2 rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base03 items-center">
            <label for="search">
                <SVGSearch class="size-6"/>
            </label>
            <input
                autocomplete="off"
                id="search"
                name="search"
                placeholder="Search"
                on:keydown=on_search
                node_ref=search_input
                class="w-full max-w-[20rem] "
                />
        </div>
    }
}
// value=value
// <form autocomplete="off" >
//             // <div
//     //     contenteditable=true
//     //     id="search"
//     //     on:keydown=on_search
//     //     class="w-full max-w-[20rem] rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base03"
//     //     >
//     //     {value}
//     // </div>
// </form>
// value=value
// placeholder="Search"
// <input type="submit"/>
// <input
//     id="search"
//     placeholder="Search"
//     value=value
//     on:keydown=on_search
//     class="w-full max-w-[20rem] rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base03"
//     />
