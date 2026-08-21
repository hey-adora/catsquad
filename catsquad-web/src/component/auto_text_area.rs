use crate::hook::Mutation;
use catsquad_log::prelude::*;
use catsquad_web_utils::prelude::MutationObserverOptions;
use leptos::{html, prelude::*};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, HtmlTextAreaElement};

#[component]
pub fn AutoTextArea(
    #[prop(optional, into)] node_ref: Option<NodeRef<html::Textarea>>,
    #[prop(optional, into)] placeholder: Option<Callback<(), String>>,
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] track: Option<Callback<()>>,
    #[prop(optional, into)] on_input: Option<Callback<HtmlTextAreaElement>>,
    #[prop(optional, into)] on_focusout: Option<Callback<HtmlTextAreaElement>>,
    #[prop(optional, into)] on_enter: Option<Callback<HtmlTextAreaElement>>,
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

    let fix_height = move || {
        trace!("AutoTextArea fix height triggered");
        let Some(input): Option<HtmlTextAreaElement> = input.get_untracked() else {
            return;
        };
        let scroll_height = input.scroll_height() as f64;

        if min_height >= scroll_height {
            return;
        }
        height.set(scroll_height);
    };

    let fn_on_change = move || {
        trace!("AutoTextArea input triggered");
        let Some(input): Option<HtmlTextAreaElement> = input.get_untracked() else {
            return;
        };
        if let Some(on_input) = on_input {
            on_input.run(input.clone());
        }
        fix_height();
    };
    let mutation = Mutation::new(move |a, b| {
        trace!("AutoTextArea mutation triggered");
        let Some(target) = input.get_untracked().map(|v| Into::<HtmlElement>::into(v)) else {
            return;
        };

        fix_height();
        b.disconnect();
    });

    Effect::new(move || {
        trace!("AutoTextArea effect triggered");
        let Some(target) = input.get().map(|v| Into::<HtmlElement>::into(v)) else {
            return;
        };

        // input.track();
        // fn_on_change();
        mutation.observe_only(
            target,
            MutationObserverOptions::new()
                .character_data()
                .set_child_list()
                .subtree(),
        );
    });

    // Effect::new(move || {
    //     if let Some(f) = track {
    //         f.run(());
    //     }
    //     let Some(input): Option<HtmlTextAreaElement> = input.get_untracked() else {
    //         return;
    //     };
    //     let scroll_height = input.scroll_height() as f64;

    //     if min_height >= scroll_height {
    //         return;
    //     }

    //     height.set(scroll_height);
    // });

    let on_focusout = move |_e: web_sys::FocusEvent| {
        trace!("AutoTextArea focusout triggered");
        let Some(input): Option<HtmlTextAreaElement> = input.get_untracked() else {
            return;
        };
        if let Some(f) = on_focusout {
            f.run(input.clone());
        }
    };

    let on_enter = move |e: web_sys::KeyboardEvent| {
        let key = e.key();
        trace!("key pressed {key}");
        if key.to_lowercase() != "enter" {
            return;
        }
        e.prevent_default();

        trace!("AutoTextArea focusout triggered");
        let Some(input): Option<HtmlTextAreaElement> = input.get_untracked() else {
            return;
        };
        if let Some(f) = on_enter {
            f.run(input.clone());
        }
    };

    // TODO maybe convert px to rem
    view! {
        <textarea
            placeholder=placeholder_fn
            node_ref=input
            id=id_fn
            on:input=move |_| fn_on_change()
            on:focusout=on_focusout
            on:keydown=on_enter
            style:height=move|| format!("{}px", height.get())
            class=class_fn
            >{children()}</textarea>
    }
}
