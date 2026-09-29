use super::{EditSaveCancel, LengthCounter};
use crate::{
    AutoTextArea, Errs, PageState, TextEditor,
    hook::Spawner,
    page::{create_client, post::post_api::PostApi},
};
use catsquad_log::prelude::*;
use catsquad_shared::MAX_POST_DESCRIPTION_LENGTH;
use leptos::{html, prelude::*};
use web_sys::HtmlTextAreaElement;

#[component]
pub fn PostDescription(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let page = PageState::get();
    let description_input_editor = NodeRef::<html::Textarea>::new();
    let when_is_owner = move || page.acc_username() == post_api.author_username.get();

    let edit_description_save = move || {
        let (post_id, Some(new_description)) = (
            post_id.get(),
            description_input_editor
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        spawner.spawn(async move {
            let client = create_client();
            post_api
                .update_description(&client, post_id, new_description)
                .await;
        });
    };

    view! {

        <TextEditor
            id_prefix="decription"
            title="Decription"
            text=post_api.description
            text_length=post_api.live_description_length
            is_owned=when_is_owner
            edit_mode_enabled=post_api.update_description_mode
            max_length=MAX_POST_DESCRIPTION_LENGTH
            errors=post_api.err_description
            on_save=edit_description_save
            node_ref=description_input_editor
            />
    }
}
