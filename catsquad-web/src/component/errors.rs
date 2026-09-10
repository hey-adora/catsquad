use leptos::prelude::*;

#[component]
pub fn Errs(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] error: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let when_is_full = move || !error.with(|v| v.is_empty());
    let errors = move || {
        error
            .get()
            .trim()
            .split("\n")
            .filter(|v| v.len() > 1)
            .map(|v| v.to_string())
            .map(move |v: String| view! { <li>{v}</li> })
            .collect_view()
    };
    let class = move || format!("ml-[1rem] text-base08 list-disc {}", class.get());

    view! {
        <Show when=when_is_full  >
            <ul id=id class=class >
                { errors }
            </ul>
        </Show>
    }
}
