use crate::SVGSpinner;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn BtnPrimary(
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

    let class_fn = move || {
        format!(
            "flex gap-2 place-content-center rounded-xl font-medium text-[1rem] leading-[1rem] font-bold px-[1rem] py-[0.5rem]  text-base01 {} {}",
            if is_disabled_fn() {
                "bg-base03"
            } else {
                "hover:bg-base05 bg-base0D"
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
