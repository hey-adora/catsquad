use leptos::{html, prelude::*};
use web_sys::MouseEvent;

#[component]
pub fn LinkPrimary(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] link: Signal<String>,
    children: Children,
) -> impl IntoView {
    let id_fn = move || id.get();
    let class_fn = move || class.get();
    let link_fn = move || link.get();

    view! {
        <a
            id=id_fn
            href=link_fn
            class=format!("font-bold flex gap-2 place-content-center rounded-xl text-[1rem] leading-[1rem] px-[1rem] py-[0.5rem] text-base01 hover:bg-base05 bg-base0D {}", class_fn())>
            {children()}
        </a>
    }
}
