use crate::{ModalConfirm, Nav};
use component_gallery::Gallery;
use leptos::prelude::*;

pub mod api_gallery;
pub mod component_gallery;

#[component]
pub fn Index() -> impl IntoView {
    view! {
        <main class="grid grid-rows-[auto_1fr] h-screen">
            <Nav/>
            <ModalConfirm/>
            <Gallery row_height=250 />
        </main>
    }
}
