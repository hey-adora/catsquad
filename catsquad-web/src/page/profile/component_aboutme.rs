use crate::{PageState, TextEditor, hook::Spawner, page::profile::author_state::AuthorState};
use catsquad_client::XMLSender;
use catsquad_shared::MAX_ABOUTME_LENGTH;
use leptos::prelude::*;
use web_sys::HtmlTextAreaElement;

#[component]
pub fn Aboutme(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(into)] user_username: Signal<String>,
    author_state: AuthorState<XMLSender>,
    spawner: Spawner,
) -> impl IntoView {
    let aboutme_text = author_state.aboutme;
    let aboutme_edit_mode = author_state.aboutme_edit_mode;
    let aboutme_err = author_state.err_aboutme;
    let aboutme_length = author_state.live_aboutme_length;
    let aboutme_input = NodeRef::new();
    let aboutme_save_fn = {
        let author_state = author_state.clone();
        move || {
            let author_state = author_state.clone();
            let Some(new_aboutme) = aboutme_input
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value())
            else {
                return;
            };
            spawner.spawn(async move {
                author_state.update_aboutme(new_aboutme).await;
            });
        }
    };
    let page = PageState::get();
    let when_is_owner = move || page.acc_username() == user_username.get();

    view! {
        <TextEditor
            class=class
            id_prefix="aboutme"
            title="Aboutme"
            text=aboutme_text
            text_length=aboutme_length
            is_owned=when_is_owner
            edit_mode_enabled=aboutme_edit_mode
            max_length=MAX_ABOUTME_LENGTH
            errors=aboutme_err
            on_save=aboutme_save_fn.clone()
            node_ref=aboutme_input
            spawner
            />
    }
}
