use leptos::prelude::*;

#[component]
pub fn LengthCounter(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] counter_current: Signal<usize>,
    #[prop(optional, into)] counter_max: Signal<usize>,
) -> impl IntoView {
    let class = move || {
        format!(
            "{} {}",
            if counter_current.get() >= counter_max.get() {
                "text-base08"
            } else {
                ""
            },
            class.get()
        )
    };

    let counter_current = move || counter_current.get();
    let counter_max = move || counter_max.get();

    view! {
        <div class=class>
            <span id="description_length">{counter_current}</span>"/"{counter_max}
        </div>
    }
}
