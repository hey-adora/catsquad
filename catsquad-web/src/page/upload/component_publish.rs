use super::UploadState;
use super::component_edit_text::ValidState;
use crate::BtnPrimary;
use leptos::prelude::*;

#[component]
pub fn Publish(
    // #[prop(optional, into)] class: Option<Callback<(), String>>,
    // #[prop(optional, into)] is_valid: Option<Callback<(), ValidState>>,
    // #[prop(optional)] required: bool,
    // children: Children,
    upload: UploadState,
) -> impl IntoView {
    // let is_valid = move || is_valid.map(|v| v.run(())).unwrap_or_default();
    // let fn_class = move || class.map(|v| v.run(())).unwrap_or_default();

    // let container_color = move || match is_valid() {
    //     ValidState::Valid => "border-base0B bg-base0B/5",
    //     ValidState::Error => "border-base08 bg-base08/5",
    //     ValidState::Empty => match required {
    //         true => "border-base08 bg-base08/5",
    //         false => "border-base0A bg-base0A/5",
    //     },
    // };

    view! {
        <div class="flex">
            <BtnPrimary class=move||"ml-auto">"Publish"</BtnPrimary>
        </div>
    }
}
