use crate::Nav;
use catsquad_log::prelude::*;
use leptos::prelude::*;

#[component]
pub fn Index() -> impl IntoView {
    view! {
        <main class="grid grid-rows-[auto_1fr] h-screen">
            // "hello from index"
            <Nav/>
            "index"
            // <Gallery row_height=250 />
        </main>
    }
}
