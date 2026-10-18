use super::index::component_gallery::Gallery;
use crate::{ModalConfirm, Nav, PageState};
use leptos::prelude::*;

#[component]
pub fn Followed() -> impl IntoView {
    let page = PageState::get();
    // let user_username = move || page.acc_username.get();

    view! {
        <main class="grid grid-rows-[auto_1fr] h-screen">
            <Nav/>
            <ModalConfirm/>
            <Gallery followed_by_username=page.acc_username row_height=250 />
        </main>
    }
}
