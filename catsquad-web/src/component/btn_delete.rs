use crate::Btn;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn BtnDelete(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] is_loading: Option<Callback<(), bool>>,
    #[prop(optional, into)] disabled: Option<Callback<(), bool>>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let on_click_handler = move |e| {
        if let Some(on_click) = on_click {
            on_click.run(e);
        }
    };
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let is_loading_fn = move || is_loading.map(|v| v.run(())).unwrap_or_default();
    let disabled_fn = move || disabled.map(|v| v.run(())).unwrap_or_default();
    let is_disabled_fn = move || is_loading_fn() || disabled_fn();

    let class_on_disable = move || "bg-base03 font-bold text-base01";
    let class_on_active = move || "hover:bg-base05 bg-base08 font-bold text-base01";

    view! {
        <Btn
            id=id_fn
            disabled=is_disabled_fn
            on:click=on_click_handler
            class=class_fn
            class_on_disable=class_on_disable
            class_on_active=class_on_active
            >
            {children()}
        </Btn>
    }
}
