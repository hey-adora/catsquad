use crate::{Btn, BtnSize};
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn BtnSecondary(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] is_loading: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] modal_enable: Signal<bool>,
    #[prop(optional, into)] modal_title: Signal<String>,
    #[prop(optional, into)] modal_text: Signal<String>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] size: Signal<BtnSize>,
    children: Children,
) -> impl IntoView {
    let on_click_handler = move |e| {
        if let Some(on_click) = on_click {
            on_click.run(e);
        }
    };
    let id_fn = move || id.get();
    let class_fn = move || class.get();

    view! {
        <Btn
            secondary=true
            modal_enable
            modal_title
            modal_text
            id=id_fn
            is_loading=is_loading
            disabled=disabled
            on_click=on_click_handler
            class=class_fn
            size
            >
            {children()}
        </Btn>
    }
}
