use leptos::{html, prelude::*};
use web_sys::MouseEvent;

#[component]
pub fn BtnSecondary(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
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

    view! {
        <button id=id_fn  on:click=on_click_handler class=format!("text-center rounded-xl font-medium text-[1rem] font-bold px-[1rem] pt-[0.1rem] hover:bg-base0D bg-base03 text-base05 {}", class_fn())>
            {children()}
        </button>
    }
}
