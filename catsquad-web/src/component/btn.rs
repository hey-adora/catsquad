use crate::SVGSpinner;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn Btn(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] is_loading: Option<Callback<(), bool>>,
    #[prop(optional, into)] disabled: Option<Callback<(), bool>>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] class_on_disable: Option<Callback<(), String>>,
    #[prop(optional, into)] class_on_active: Option<Callback<(), String>>,
    children: Children,
) -> impl IntoView {
    let on_click_handler = move |e| {
        if let Some(on_click) = on_click {
            on_click.run(e);
        }
    };
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let class_on_disable = move || class_on_disable.map(|v| v.run(())).unwrap_or_default();
    let class_on_active = move || class_on_active.map(|v| v.run(())).unwrap_or_default();
    let is_loading_fn = move || is_loading.map(|v| v.run(())).unwrap_or_default();
    let disabled_fn = move || disabled.map(|v| v.run(())).unwrap_or_default();

    let is_disabled_fn = move || is_loading_fn() || disabled_fn();

    let class_fn = move || {
        format!(
            "flex gap-2 place-content-center rounded-xl text-[1rem] leading-[1rem] px-[1rem] py-[0.5rem]  {} {}",
            if is_disabled_fn() {
                class_on_disable()
            } else {
                class_on_active()
            },
            class_fn()
        )
    };

    view! {
        <button
            id=id_fn
            disabled=is_disabled_fn
            on:click=on_click_handler
            class=class_fn>
            <Show when=is_loading_fn>
                <SVGSpinner class=move||"size-4"/>
            </Show>
            {children()}
        </button>
    }
}
