use super::index::component_gallery::Gallery;
use crate::hook::Spawner;
use crate::page::create_client;
use crate::page::profile::author_state::AuthorState;
use crate::page_state::CurrentRoute;
use crate::{Nav, PageState};
use leptos::prelude::*;
use leptos_router::hooks::use_location;
use leptos_router::params::Params;

pub mod author_state;
pub mod component_aboutme;
pub mod component_author;
pub mod component_support;

use component_aboutme::Aboutme;
use component_author::ProfileAuthor;
use component_support::Support;

#[derive(Params, PartialEq, Clone, Default)]
pub struct ProfileParams {
    pub username: String,
}

#[component]
pub fn Profile() -> impl IntoView {
    let page = PageState::get();
    let params = page.create_page_profile_params_hook();
    let username = move || params.get().username;
    let spawner = Spawner::new();

    let page = PageState::get();
    let location = use_location();
    let current_route = page.current_route(location);
    let when_tab_paws = move || current_route.get().is_profile_paws();
    let when_tab_support = move || current_route.get().is_profile_support();
    let when_tab_aboutme = move || current_route.get().is_profile_about_me();

    let liked_by_username = move || {
        if when_tab_paws() {
            username()
        } else {
            String::new()
        }
    };

    let author_state = AuthorState::new(create_client());
    let author_state_clone1 = author_state.clone();
    Effect::new({
        let author_state = author_state.clone();
        move || {
            let username = username();
            let author_state = author_state.clone();

            spawner.spawn(async move {
                author_state.init(username).await;
            });
        }
    });

    let class_gallery = move || {
        format!(
            "sm:col-start-1 sm:col-row-1 {}",
            match current_route.get() {
                CurrentRoute::ProfileSupport | CurrentRoute::ProfileAboutMe => "hidden sm:block",
                _ => "",
            }
        )
    };

    view! {
        <main class="grid grid-rows-[auto_1fr] h-screen">
            <Nav />
            <div class="grid grid-rows-[auto_1fr] sm:grid-rows-[1fr] sm:grid-cols-[1fr_auto] ">
                <ProfileAuthor author_state=author_state.clone() spawner user_username=username />
                <Show when=when_tab_support>
                    <Support
                        class="px-2 sm:col-start-1 sm:col-row-1 sm:hidden"
                        user_username=username
                        spawner
                        author_state=author_state.clone()
                    />
                </Show>
                <Show when=when_tab_aboutme>
                    <Aboutme
                        class="px-2 sm:col-start-1 sm:col-row-1 sm:hidden"
                        user_username=username
                        spawner
                        author_state=author_state_clone1.clone()
                    />
                </Show>
                <Gallery class=class_gallery liked_by_username author_username=username row_height=250 />
            </div>
        </main>
    }
}
