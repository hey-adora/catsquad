use crate::{Btn, BtnSize};
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn BtnPrimary(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] is_loading: Signal<bool>,
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
    let id_fn = move || id.get();
    let class_fn = move || class.get();
    let is_loading_fn = move || is_loading.get();
    let disabled_fn = move || disabled.get();
    let is_disabled_fn = move || is_loading_fn() || disabled_fn();

    let class_on_disable = move || "bg-base03 font-bold text-base01";
    let class_on_active = move || "hover:bg-base05 bg-base0D font-bold text-base01";

    view! {
        <Btn
            id=id_fn
            disabled=is_disabled_fn
            on:click=on_click_handler
            class=class_fn
            class_on_disable=class_on_disable
            class_on_active=class_on_active
            size
            >
            {children()}
        </Btn>
    }
}
