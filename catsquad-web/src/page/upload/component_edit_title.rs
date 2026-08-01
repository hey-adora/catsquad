use super::component_edit_text::TextEdit;
use super::upload_state::UploadState;
use crate::{hook::Spawner, page::create_client};
use catsquad_log::prelude::*;
use catsquad_shared::MAX_POST_TITLE_LENGTH;
use catsquad_web_utils::prelude::*;
use leptos::prelude::*;

#[component]
pub fn TitleEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
    let on_update = move || {
        spawner.spawn(async move {
            let time = time_now_ns();
            let client = create_client();
            trace!("auto save title - running");
            upload.update_title(time, &client).await;
        });
    };
    let on_set = move |value| {
        let time = time_now_ns();
        upload.set_title(time, value);
    };
    view! {
        <TextEdit
            on_update
            on_set
            max_length=MAX_POST_TITLE_LENGTH
            min_height_rem=2
            title="Title"
            saved_metadata=upload.title_saved
            input_text=upload.title
            errors=upload.err_title/>
    }
}
