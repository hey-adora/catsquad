use super::component_edit_text::TextEdit;
use super::upload_state::UploadState;
use crate::{hook::Spawner, page::create_client};
use catsquad_log::prelude::*;
use catsquad_shared::MAX_POST_TAGS_LENGTH;
use catsquad_web_utils::prelude::*;
use leptos::prelude::*;

#[component]
pub fn TagsEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
    let on_update = move || {
        spawner.spawn(async move {
            let time = time_now_ns();
            let client = create_client();
            trace!("auto save tags - running");
            upload.update_tags(time, &client).await;
        });
    };
    let on_set = move |value| {
        let time = time_now_ns();
        upload.set_tags(time, value);
    };
    view! {
        <TextEdit
            on_update
            on_set
            max_length=MAX_POST_TAGS_LENGTH
            min_height_rem=8
            title="Tags"
            saved_metadata=upload.tags_saved
            input_text=upload.tags
            errors=upload.err_tags/>
    }
}
