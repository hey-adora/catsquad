use leptos::prelude::*;

#[component]
pub fn LengthCounter(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] counter_current: Option<Callback<(), usize>>,
    #[prop(optional, into)] counter_max: Option<Callback<(), usize>>,
) -> impl IntoView {
    let counter_current_fn = move || counter_current.map(|v| v.run(())).unwrap_or_default();
    let counter_max_fn = move || counter_max.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();

    view! {
        <div class=move || format!("{} {}", if counter_current_fn() > counter_max_fn() {"text-base08"} else {""}, class_fn())>
            <span id=move||id_fn()>{counter_current_fn}</span>"/"{counter_max_fn}
        </div>
    }
}
