use leptos::prelude::*;

#[component]
pub fn Display(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] when: Option<Callback<(), bool>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    children: Children,
) -> impl IntoView {
    let when_fn = move || when.map(|v| v.run(())).unwrap_or_default();
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();

    view! {
        <div id=id_fn class=move||format!("{} {}", class_fn(), if when_fn() {""} else {"hidden"})  >
            { children() }
        </div>
    }
}
