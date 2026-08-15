use leptos::{html, prelude::*};
use web_sys::MouseEvent;

#[component]
pub fn LinkPrimary(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] link: Option<Callback<(), String>>,
    children: Children,
) -> impl IntoView {
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let link_fn = move || link.map(|v| v.run(())).unwrap_or_default();

    view! {
        <a
            id=id_fn
            href=link_fn
            class=format!("flex gap-2 place-content-center rounded-xl font-medium text-[1rem] leading-[1rem] font-bold px-[1rem] py-[0.5rem] text-base01 hover:bg-base05 bg-base0D {}", class_fn())>
            {children()}
        </a>
    }
}
