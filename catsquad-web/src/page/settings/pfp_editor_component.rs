use super::user_pfp_state::UserPfpState;
use crate::{Errs, PageState, PfpAcc, hook::Spawner, page::create_client};
use catsquad_log::prelude::*;
use catsquad_web_utils::file::GetFiles;
use leptos::prelude::*;
use web_sys::HtmlInputElement;

#[component]
pub fn PfpEditor() -> impl IntoView {
    let page = PageState::get();
    let pfp_state =
        UserPfpState::new_with_state(create_client(), page.acc_pfp_hash, page.acc_pfp_proccesed);
    let input_image = NodeRef::new();
    let spawner = Spawner::new();

    let on_file_change = move |e| {
        trace!("on_pfp_change");
        let Some(new_img) = (input_image.get_untracked() as Option<HtmlInputElement>)
            .and_then(|f: HtmlInputElement| f.files())
            .and_then(|f| f.get_files().first().cloned())
        else {
            warn!("pfp upload canceled, failed to get files");
            return;
        };

        spawner.spawn(async move {
            pfp_state.update_pfp(new_img).await;
        });
    };

    let is_loading = move || spawner.is_busy.get();

    let class_on_disable = "bg-base02 text-base01";
    let class_on_active = "hover:bg-base0D bg-base03 text-base05";
    let class = move || {
        format!(
            "px-2 py-1 rounded-xl {}",
            if is_loading() {
                class_on_disable
            } else {
                class_on_active
            }
        )
    };

    view! {
        <div class="flex flex-col gap-4 place-items-center">
            <PfpAcc no_link=true is_loading class="size-[10rem]"/>
            <Errs error=pfp_state.err_general />
            <label
                for="pfp_upload_input"
                class=class
                >
                "Upload"
            </label>
            <input disabled=is_loading class="absolute z-[-1] opacity-0" on:change=on_file_change type="file" id="pfp_upload_input" name="image" node_ref=input_image />
        </div>
    }
}
