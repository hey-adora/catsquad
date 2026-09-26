use super::index::component_gallery::Gallery;
use crate::{Nav, PageState};
use catsquad_log::prelude::*;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq, Clone, Default)]
pub struct ProfileParams {
    pub username: String,
}

#[component]
pub fn Profile() -> impl IntoView {
    let page = PageState::get();
    let params = page.create_page_profile_params_hook();
    let username = move || params.get().username;

    // let acc_username = move || page.acc_username();

    view! {
        <main class="grid grid-rows-[auto_1fr] h-screen">
            <Nav/>
            <Gallery username=username row_height=250 />

        </main>
    }
}
