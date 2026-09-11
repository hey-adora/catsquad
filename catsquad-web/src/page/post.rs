use crate::Nav;
use crate::hook::Spawner;
use crate::page::create_client;
use crate::page::post::post_like_state::PostLikeState;
use leptos::Params;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_params};
use leptos_router::params::Params;
use post_api::PostApi;

mod comments_api;
mod comments_basic;
mod component_comment;
mod component_comments;
mod component_edit_btn;
mod component_favorite;
mod component_length_counter;
mod component_post_controls;
mod component_post_description;
mod component_post_imgs;
mod component_post_previews;
mod component_post_profile;
mod component_post_selected_img;
mod component_post_stats;
mod component_post_tags;
mod component_post_title;
mod post_api;
mod post_like_state;

use component_comments::Comments;
use component_edit_btn::EditSaveCancel;
use component_favorite::Favorite;
use component_length_counter::LengthCounter;
use component_post_controls::PostControls;
use component_post_description::PostDescription;
use component_post_imgs::PostImgs;
use component_post_previews::PostPreviews;
use component_post_profile::PostProfile;
use component_post_selected_img::PostSelectedImg;
use component_post_stats::PostStats;
use component_post_tags::PostTags;
use component_post_title::PostTitle;

#[derive(Params, PartialEq, Clone)]
pub struct PostParams {
    pub username: Option<String>,
    pub post: Option<String>,
}

#[component]
pub fn Post() -> impl IntoView {
    // TODO add max limit for description and tags and other stuff

    let param = use_params::<PostParams>();
    let param_post_id = Memo::new(move |_| {
        param
            .read()
            .as_ref()
            .ok()
            .and_then(|v| v.post.clone())
            .and_then(|v| i64::from_str_radix(&v, 10).ok())
    });
    let post_id = move || param_post_id.get().unwrap_or_default();

    let main_ref = NodeRef::new();
    let spawner_post = Spawner::new();
    let post_api = PostApi::new();
    // let post_like_api = PostLikeState::new(create_client());

    Effect::new(move || {
        let Some(post_id) = param_post_id.get() else {
            return;
        };

        spawner_post.spawn(async move {
            let client = create_client();
            post_api.get(&client, post_id).await;
            // post_like_api.init(post_id).await;
        });
    });

    let when_not_found = move || post_api.api_state.get().is_not_found();
    let when_deleted = move || post_api.api_state.get().is_deleted();
    let when_show = move || {
        let state = post_api.api_state.get();
        state.is_normal() || state.is_loading()
    };

    view! {
        <main node_ref=main_ref class="relative font-hi grid gap-6 grid-rows-[auto_1fr] h-screen text-base05">
            <Nav/>

            <Show when=when_not_found >
                <div class="grid place-items-center text-[1.5rem] ">
                    "Not Found"
                </div>
            </Show>

            <Show when=when_deleted >
                <div class="grid place-items-center text-[1.5rem] ">
                    "deleted"
                </div>
            </Show>

            <Show when=when_show >
                <div class="flex flex-col lg:grid grid-rows-[auto_1fr] grid-cols-[2fr_1fr] lg:max-h-[calc(100vh-3rem)] gap-2  md:gap-6">
                    <PostControls spawner=spawner_post post_api post_id/>
                    <PostSelectedImg post_api />
                    <PostImgs post_api />
                    <div class="flex flex-col gap-2 md:gap-6 px-4 md:px-6  lg:overflow-y-scroll">
                        <PostPreviews spawner=spawner_post post_api post_id/>
                        <PostStats post_api/>
                        <div class="flex flex-col gap-2">
                            <PostTitle spawner=spawner_post post_api post_id />
                            <PostProfile spawner=spawner_post post_api post_id />
                        </div>
                        <PostDescription spawner=spawner_post post_api post_id/>
                        <PostTags spawner=spawner_post post_api post_id/>
                        <Comments post_id />
                    </div>
                </div>
            </Show>


            // TODO probably change 1fr to fixed size or auto or minmax bs
        </main>
    }
}
