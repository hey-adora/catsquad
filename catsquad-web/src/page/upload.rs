use crate::{
    Nav, PageState,
    hook::{PostImagesState, Spawner},
    page::{
        create_client,
        post::post_api::PostApi,
        upload::upload_state::{UploadState, UploadStateStage},
    },
};
use catsquad_web_utils::time::{time_now_micro, time_now_ns};
use leptos::prelude::*;

// const AUTO_SAVE_TIME: u128 = 3000000000; // 3s
const AUTO_SAVE_TIME: u64 = 3000000; // 3s

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
    let time = time_now_micro();
    let page = PageState::get();
    let spawner = Spawner::new();
    let upload = UploadState::new(time);
    // let post_api = PostApi::new();

    let post_author_username = move || page.acc_username();
    let post_id = move || upload.post_id.get_value();
    let post_files = upload.files;
    // let ab = post_api.imgs;

    Effect::new(move || {
        spawner.spawn(async move {
            let client = create_client();
            upload.init(&client).await;
            // let post_id = post_id();
            // post_api.init(&client, 616).await;
        });
    });

    // Effect::new(move || {
    //     let post_id = post_id();
    //     if post_id == 0 {
    //         return;
    //     }
    //     spawner.spawn(async move {
    //         let client = create_client();
    //         post_api.init(&client, post_id).await;
    //     });
    // });

    view! {
        <main>
            <Nav/>
            <div class="flex flex-col gap-4 max-w-[25rem] mx-auto" >
                <Publish upload/>
                <TitleEdit upload/>
                <ImagesEdit author_username=post_author_username post_id=post_id images=post_files />
                <DescriptionEdit upload/>
                <TagsEdit upload/>
            </div>
        </main>
    }
}

// trace!(
//     "auto save title - checking - {} - {} >= {} = elapsed({}) && saved({})",
//     time, meta_data.set_at, AUTO_SAVE_TIME, elapsed, meta_data.saved
// );
