use leptos::{html, prelude::*};
use web_sys::MouseEvent;

#[component]
pub fn LinkSecondary(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] link: Signal<String>,
    children: Children,
) -> impl IntoView {
    let id_fn = move || id.get();
    let class_fn = move || class.get();
    let link_fn = move || link.get();

    view! {
        <a id=id_fn  href=link_fn class=format!("text-center rounded-xl font-medium text-[1rem] leading-[1rem] font-bold px-[1rem] py-[0.5rem] hover:bg-base0D bg-base03 text-base05 {}", class_fn())>
            {children()}
        </a>
    }
}
