use leptos::prelude::*;

#[component]
pub fn ErrGeneral(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] error: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
) -> impl IntoView {
    let error_fn = move || error.map(|v| v.run(())).unwrap_or_default();
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();

    view! {
        <Show when=move || !error_fn().is_empty()  >
            <ul id=id_fn class=move|| format!("text-base08 text-center {}", class_fn())>
                {move || error_fn().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
            </ul>
        </Show>
    }
}
