use crate::Nav;
use catsquad_log::prelude::*;
use component_gallery::Gallery;
use leptos::prelude::*;

mod api_gallery;
mod component_gallery;

#[component]
pub fn Index() -> impl IntoView {
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
