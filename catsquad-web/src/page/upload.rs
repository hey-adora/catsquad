use crate::{
    Nav,
    hook::Spawner,
    page::{create_client, upload::upload_state::UploadState},
};
use catsquad_web_utils::time::time_now_ns;
use leptos::prelude::*;

const AUTO_SAVE_TIME: u128 = 3000000000; // 3s

pub mod component_edit_area;
pub mod component_edit_description;
pub mod component_edit_files;
pub mod component_edit_tags;
pub mod component_edit_text;
pub mod component_edit_title;
pub mod component_publish;
pub mod upload_state;

use component_edit_description::DescriptionEdit;
use component_edit_files::ImagesEdit;
use component_edit_tags::TagsEdit;
use component_edit_title::TitleEdit;
use component_publish::Publish;

#[component]
pub fn Upload() -> impl IntoView {
    let time = time_now_ns();
    let spawner = Spawner::new();
    let upload = UploadState::new(time);

    Effect::new(move || {
        spawner.spawn(async move {
            let client = create_client();
            upload.init(&client).await;
        });
    });

    view! {
        <Nav/>
        <div class="flex flex-col gap-4 max-w-[25rem] mx-auto" >
            <Publish/>
            <TitleEdit upload/>
            <ImagesEdit upload/>
            <DescriptionEdit upload/>
            <TagsEdit upload/>
        </div>
    }
}

// trace!(
//     "auto save title - checking - {} - {} >= {} = elapsed({}) && saved({})",
//     time, meta_data.set_at, AUTO_SAVE_TIME, elapsed, meta_data.saved
// );
