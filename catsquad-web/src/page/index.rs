use crate::{Nav, hook::ScrollCorrection, page::index::api_gallery::GalleryApi};
use catsquad_log::prelude::*;
use component_gallery::Gallery;
use leptos::prelude::*;

pub mod api_gallery;
pub mod component_gallery;

#[component]
pub fn Index() -> impl IntoView {
    // let scroll_correction = ScrollCorrection::new();
    // let gallery_api = GalleryApi::new(scroll_correction.clone());

    view! {
        <main class="grid grid-rows-[auto_1fr] h-screen">
            <Nav/>
            <Gallery row_height=250 />

        </main>
    }
}
// // "hello from index"
// <Nav/>
// "index"
// // <Gallery row_height=250 />
