use crate::Nav;
use catsquad_log::prelude::*;
use leptos::prelude::*;

#[component]
pub fn Post() -> impl IntoView {
    view! {
        <main class="grid grid-rows-[auto_1fr]">
            <Nav/>
            "post"
        </main>
    }
}
