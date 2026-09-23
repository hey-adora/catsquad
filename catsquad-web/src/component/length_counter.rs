use leptos::prelude::*;

#[component]
pub fn LengthCounter(
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] counter_current: Signal<usize>,
    #[prop(optional, into)] counter_max: Signal<usize>,
) -> impl IntoView {
    let counter_current_fn = move || counter_current.get();
    let counter_max_fn = move || counter_max.get();
    let id_fn = move || id.get();

    let class_fn = move || {
        format!(
            "{} {}",
            if counter_current_fn() > counter_max_fn() {
                "text-base08"
            } else {
                ""
            },
            class.get()
        )
    };

    view! {
        <div class=class_fn >
            <span id=id_fn>{counter_current_fn}</span>"/"{counter_max_fn}
        </div>
    }
}
