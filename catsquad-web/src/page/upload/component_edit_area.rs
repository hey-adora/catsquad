use super::component_edit_text::ValidState;
use leptos::prelude::*;

#[component]
pub fn EditArea(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] is_valid: Signal<ValidState>,
    #[prop(optional)] required: bool,
    children: Children,
) -> impl IntoView {
    let container_color = move || match is_valid.get() {
        ValidState::Valid => "border-base0B bg-base0B/5",
        ValidState::Error => "border-base08 bg-base08/5",
        ValidState::Empty => match required {
            true => "border-base08 bg-base08/5",
            false => "border-base0A bg-base0A/5",
        },
    };

    let fn_class = move || {
        format!(
            "border-2 rounded-lg px-3 py-2 {} {}",
            container_color(),
            class.get()
        )
    };

    view! {
        <div class=fn_class>
            {children()}
        </div>
    }
}
