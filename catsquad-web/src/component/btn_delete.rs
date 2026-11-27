use crate::{Btn, BtnSize};
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn BtnDelete(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] is_loading: Signal<bool>,
    #[prop(optional, into)] modal_enable: Signal<bool>,
    #[prop(optional, into)] modal_title: Signal<String>,
    #[prop(optional, into)] modal_text: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] size: Signal<BtnSize>,
    children: Children,
) -> impl IntoView {
    let on_click_handler = move |e| {
        if let Some(on_click) = on_click {
            on_click.run(e);
        }
    };
    let is_disabled_fn = move || is_loading.get() || disabled.get();

    view! {
        <Btn
            danger=true
            size
            id=move || id.get()
            disabled=is_disabled_fn
            on_click=on_click_handler
            class=move || class.get()
            modal_enable
            modal_title
            modal_text
            >
            {children()}
        </Btn>
    }
}
