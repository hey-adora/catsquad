use catsquad_log::prelude::*;
use leptos::{html, prelude::*};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, HtmlTextAreaElement};

#[component]
pub fn AutoTextArea(
    #[prop(optional, into)] node_ref: Option<NodeRef<html::Textarea>>,
    #[prop(optional, into)] placeholder: Option<Callback<(), String>>,
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] on_input: Option<Callback<(HtmlTextAreaElement)>>,
    #[prop(default = 500.0)] min_height: f64,
    children: Children,
) -> impl IntoView {
    let id_fn = move || {
        id.map(|v| v.run(()))
            .unwrap_or_else(|| "auto-text-area".to_string())
    };
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let placeholder_fn = move || placeholder.map(|v| v.run(())).unwrap_or_default();

    let height = RwSignal::new(min_height);
    let input = node_ref.unwrap_or_else(|| NodeRef::new());
    let on_change = move || {
        let Some(input): Option<HtmlTextAreaElement> = input.get_untracked() else {
            return;
        };
        if let Some(on_input) = on_input {
            on_input.run(input.clone());
        }
        let scroll_height = input.scroll_height() as f64;

        if min_height >= scroll_height {
            return;
        }

        height.set(scroll_height);
    };

    Effect::new(move || {
        input.track();
        trace!("AutoTextArea effect triggered");
        on_change();
    });

    // TODO maybe convert px to rem
    view! {
        <textarea
            placeholder=placeholder_fn
            node_ref=input
            id=id_fn
            on:input=move |_| on_change()
            style:height=move|| format!("{}px", height.get())
            class=class_fn
            >{children()}</textarea>
    }
}
