use super::UploadState;
use super::component_edit_text::ValidState;
use crate::BtnPrimary;
use crate::hook::Spawner;
use crate::page::create_client;
use crate::page::upload::upload_state::UploadStateStage;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;

#[component]
pub fn Publish(
    // #[prop(optional, into)] is_valid: Option<Callback<(), ValidState>>,
    // #[prop(optional)] required: bool,
    // children: Children,
    // #[prop(into)] on_activate: Callback<()>,
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
    let spawner = Spawner::new();
    let navigate = use_navigate();
    let on_click = move |_e| {
        let navigate = navigate.clone();
        spawner.spawn(async move {
            // on_activate.run(());
            upload.stage.update(|v| {
                *v = UploadStateStage::Activating;
            });
            let client = create_client();
            let Some(link) = upload.update_state_active(&client).await else {
                upload.stage.update(|v| {
                    *v = UploadStateStage::Err;
                });
                return;
            };
            // on_activate.run(());
            navigate(&link, NavigateOptions::default());
        });
    };

    view! {
        <div class="flex">
            <BtnPrimary on_click class=move||"ml-auto">"Publish"</BtnPrimary>
        </div>
    }
}
