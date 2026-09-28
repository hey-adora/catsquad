use super::index::component_gallery::Gallery;
use crate::hook::ScrollCorrection;
use crate::page::index::api_gallery::GalleryApi;
use crate::{Nav, PageState};
use catsquad_log::prelude::*;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

pub mod author_state;
pub mod component_author;

use component_author::ProfileAuthor;

#[derive(Params, PartialEq, Clone, Default)]
pub struct ProfileParams {
    pub username: String,
}

#[component]
pub fn Profile() -> impl IntoView {
    let page = PageState::get();
    let params = page.create_page_profile_params_hook();
    let username = move || params.get().username;

    // let scroll_correction = ScrollCorrection::new();
    // let gallery_api = GalleryApi::new(scroll_correction.clone());

    // let acc_username = move || page.acc_username();

    view! {
        <main class="grid grid-rows-[auto_1fr] h-screen">
            <Nav />
            <div class="grid grid-rows-[auto_1fr] sm:grid-rows-[1fr] sm:grid-cols-[1fr_auto] ">
                <ProfileAuthor user_username=username />
                <Gallery class="sm:col-start-1 sm:col-row-1" username=username row_height=250 />
            </div>
        </main>
    }
}
