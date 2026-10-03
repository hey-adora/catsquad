use crate::{PageState, TextEditor, hook::Spawner, page::profile::author_state::AuthorState};
use catsquad_client::XMLSender;
use catsquad_shared::MAX_SUPPORT_LENGTH;
use leptos::prelude::*;
use web_sys::HtmlTextAreaElement;

#[component]
pub fn Support(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(into)] user_username: Signal<String>,
    author_state: AuthorState<XMLSender>,
    spawner: Spawner,
) -> impl IntoView {
    let support_text = author_state.support;
    let support_edit_mode = author_state.support_edit_mode;
    let support_err = author_state.err_support;
    let support_length = author_state.live_support_length;
    let support_input = NodeRef::new();
    let support_save_fn = {
        let author_state = author_state.clone();
        move || {
            let author_state = author_state.clone();
            let Some(new_support) = support_input
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value())
            else {
                return;
            };
            spawner.spawn(async move {
                author_state.update_support(new_support).await;
            });
        }
    };
    let page = PageState::get();
    let when_is_owner = move || page.acc_username() == user_username.get();

    view! {
        <TextEditor
            class=class
            id_prefix="support"
            title="Support"
            text=support_text
            text_length=support_length
            is_owned=when_is_owner
            edit_mode_enabled=support_edit_mode
            max_length=MAX_SUPPORT_LENGTH
            errors=support_err
            on_save=support_save_fn.clone()
            node_ref=support_input
            min_height=200.0
            spawner
            />
    }
}
