use leptos::{html, prelude::*};
use web_sys::MouseEvent;

#[component]
pub fn LinkSecondary(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] link: Option<Callback<(), String>>,
    children: Children,
) -> impl IntoView {
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let link_fn = move || link.map(|v| v.run(())).unwrap_or_default();

    view! {
        <a id=id_fn  href=link_fn class=format!("text-center rounded-xl font-medium text-[1rem] leading-[1rem] font-bold px-[1rem] py-[0.5rem] hover:bg-base0D bg-base03 text-base05 {}", class_fn())>
            {children()}
        </a>
    }
}
