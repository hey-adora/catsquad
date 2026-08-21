use catsquad_log::prelude::*;
use catsquad_shared::{LINK_WEB_INDEX, link_relative_index_search};
use leptos::{html::Textarea, prelude::*};
use leptos_router::hooks::{query_signal, use_navigate};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlTextAreaElement, KeyboardEvent};

#[component]
pub fn SearchBar() -> impl IntoView {
    let search_input = NodeRef::<Textarea>::new();
    let navigate = use_navigate();
    let (get_query_tags, set_query_tags) = query_signal::<String>("tags");

    let on_search = move |e: KeyboardEvent| {
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
        let (Some(search_elm), val): (Option<HtmlTextAreaElement>, Option<String>) =
            (search_input.get(), get_query_tags.get())
        else {
            return;
        };
        if let Some(v) = val {
            search_elm.set_value(&v);
        } else {
            search_elm.set_value("");
        }
        // let val = ;
    });

    view! {
        <input
            id="search"
            placeholder="Search"
            on:keydown=on_search
            class=" rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base03"
            />
    }
}
