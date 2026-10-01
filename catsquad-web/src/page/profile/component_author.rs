use super::author_state::{AuthorStage, AuthorState};
use crate::{
    Btn, BtnSecondary, BtnSize, LinkSecondary, PageState, SVGArrowDown, SVGBell, TextEditor,
    hook::Spawner,
    page::{create_client, index::api_gallery::GalleryApi},
    page_state::CurrentRoute,
};
use catsquad_client::XMLSender;
use catsquad_shared::{
    MAX_ABOUTME_LENGTH, MAX_SUPPORT_LENGTH, link_relative_profile_aboutme,
    link_relative_profile_gallery, link_relative_profile_paws, link_relative_profile_support,
};
use catsquad_web_utils::prelude::*;
use leptos::prelude::*;
use leptos_router::hooks::use_location;
use web_sys::HtmlTextAreaElement;

#[component]
pub fn ProfileAuthor(
    #[prop(into)] user_username: Signal<String>,
    author_state: AuthorState<XMLSender>,
    spawner: Spawner,
) -> impl IntoView {
    let is_open = RwSignal::new(true);
    let support_text = author_state.support;
    let support_edit_mode = author_state.support_edit_mode;
    let support_err = author_state.err_support;
    let support_length = author_state.live_support_length;
    let support_input = NodeRef::new();
    let support_save_fn = {
        let author_state = author_state.clone();
        move || {
            let author_state = author_state.clone();
            let Some(new_support) = support_input
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value())
            else {
                return;
            };
            spawner.spawn(async move {
                author_state.update_support(new_support).await;
            });
        }
    };

    let aboutme_text = author_state.aboutme;
    let aboutme_edit_mode = author_state.aboutme_edit_mode;
    let aboutme_err = author_state.err_aboutme;
    let aboutme_length = author_state.live_aboutme_length;
    let aboutme_input = NodeRef::new();
    let aboutme_save_fn = {
        let author_state = author_state.clone();
        move || {
            let author_state = author_state.clone();
            let Some(new_aboutme) = aboutme_input
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value())
            else {
                return;
            };
            spawner.spawn(async move {
                author_state.update_aboutme(new_aboutme).await;
            });
        }
    };

    let page = PageState::get();
    let location = use_location();
    let current_route = page.current_route(location);
    let when_is_owner = move || page.acc_username() == user_username.get();
    let when_tab_gallery_mobile = move || match current_route.get() {
        CurrentRoute::ProfileGallery | CurrentRoute::Other => true,
        _ => false,
    };
    let when_tab_gallery_desktop = move || match current_route.get() {
        CurrentRoute::ProfilePaws => false,
        _ => true,
    };
    let when_tab_paws = move || current_route.get().is_profile_paws();
    let when_tab_support = move || current_route.get().is_profile_support();
    let when_tab_aboutme = move || current_route.get().is_profile_about_me();

    // let param_tab = RwQuery::<String>::new("tab");

    let author_username = author_state.username;
    let created_at = author_state.created_at;

    let author_username = move || author_username.get();
    let joined = move || micro_to_str(time_now_micro().saturating_sub(created_at.get()));
    let toggle_open = move |_| is_open.update(|v| *v = !*v);

    let when_is_open = move || is_open.get();
    let when_is_closed = move || !when_is_open();

    let link_tab_gallery = move || link_relative_profile_gallery(user_username.get());
    let link_tab_paws = move || link_relative_profile_paws(user_username.get());
    let link_tab_support = move || link_relative_profile_support(user_username.get());
    let link_tab_aboutme = move || link_relative_profile_aboutme(user_username.get());

    let class_side_link = "self-center text-[1.2rem]";
    let class_side_link_gallery = move || {
        format!(
            "{} {}",
            class_side_link,
            when_tab_gallery_desktop().or("underline")
        )
    };
    let class_side_link_paws =
        move || format!("{} {}", class_side_link, when_tab_paws().or("underline"));

    let class_side_minimized = "rounded-lg self-center text-[1.2rem] px-4 py-2 font-bold ";
    let class_side_minimized_gallery = move || {
        format!(
            "{} {}",
            class_side_minimized,
            when_tab_gallery_desktop().either("bg-base05 text-base01", "bg-base02")
        )
    };
    let class_side_minimized_paws = move || {
        format!(
            "{} {}",
            class_side_minimized,
            when_tab_paws().either("bg-base05 text-base01", "bg-base02")
        )
    };

    // let class_top_link = "self-center text-[1.2rem]";
    let class_top_link_gallery = move || format!("{}", when_tab_gallery_mobile().or("underline"));
    let class_top_link_paws = move || format!("{}", when_tab_paws().or("underline"));
    let class_top_link_support = move || format!("{}", when_tab_support().or("underline"));
    let class_top_link_aboutme = move || format!("{}", when_tab_aboutme().or("underline"));

    view! {
        <div class=" sm:hidden grid grid-rows-[auto_auto] gap-2 max-w-full overflow-hidden ">
            <div class="grid grid-cols-[auto_auto] justify-start  px-2 gap-2 mb-2 " >
                <p class="self-start text-[1rem] rounded-full h-[3.2rem] w-[3.2rem] shrink-0 bg-base05"></p>
                <div class="grid grid-rows-[auto_auto_auto] gap-1">
                    <ul class="grid grid-cols-[auto_auto_auto] ml-[1rem] whitespace-nowrap gap-[1.5rem] text-[0.8rem] list-disc">
                        <li>"followers 9999"</li>
                        <li>"paws 450"</li>
                        <li>"joined "{joined}" ago"</li>
                    </ul>
                    <div class="grid grid-cols-[auto_auto] max-w-[calc(100%-3.5rem)] justify-start items-center gap-[1.5rem] ">
                        <p class="text-ellipsis overflow-hidden shrink-0 text-[1.2rem] leading-[1.2rem] font-medium text-base0F">
                            {author_username}
                        </p>
                        <BtnSecondary size=BtnSize::Small class="grid grid-cols-[auto_auto] gap-2 place-items-center ">"Follow"<SVGBell class="size-4"/></BtnSecondary>
                    </div>
                </div>
            </div>
            <ul class="pl-2 mb-2 justify-start grid grid-cols-[auto_auto_auto_auto] list-none gap-2">
                <li><a href=link_tab_gallery class=class_top_link_gallery >"Gallery"</a></li>
                <li><a href=link_tab_paws class=class_top_link_paws >"Paws"</a></li>
                <li><a href=link_tab_support class=class_top_link_support >"Support"</a></li>
                <li><a href=link_tab_aboutme class=class_top_link_aboutme >"About Me"</a></li>
            </ul>
        </div>

        <Show when=when_is_open>
            <div class="hidden sm:grid sm:col-start-2 sm:row-start-1 grid-rows-[auto_auto_auto_auto_1fr] gap-4 px-6 py-4 mr-2 mb-2 border-base02 border-3 rounded-lg">
                <div class="flex justify-start gap-4 ">
                    <Btn class="self-start bg-base02 " on_click=toggle_open><SVGArrowDown stroke=2.5 class="size-6 rotate-90 "/></Btn>
                    <div class="flex w-full justify-center gap-6 ">
                        <a href=link_tab_gallery class=class_side_link_gallery >"Gallery"</a>
                        <a href=link_tab_paws class=class_side_link_paws >"Paws"</a>
                    </div>
                </div>
                <div class="place-items-center grid grid-cols-[auto_auto] gap-4 ">
                    <p class="self-start text-[1rem] rounded-full size-[5rem] shrink-0 bg-base05"></p>
                    <div class="grid grid-rows-[auto_auto_auto] gap-2 place-items-start" >
                        <p class="text-[1.2rem] leading-[1.2rem] font-medium text-base0F max-w-[10rem] break-all ">{ author_username }</p>
                        <BtnSecondary size=BtnSize::Normal class="grid grid-cols-[auto_auto] gap-2 place-items-center ">"Follow"<SVGBell class="size-4"/></BtnSecondary>
                        <ul class="ml-[1rem] whitespace-nowrap gap-[1.5rem] text-[1rem] list-disc">
                            <li>"followers 9999"</li>
                            <li>"paws 450"</li>
                            <li>"joined "{joined}" ago"</li>
                        </ul>
                    </div>
                </div>
                <TextEditor
                    class="max-w-[20rem]"
                    id_prefix="support"
                    title="Support"
                    text=support_text
                    text_length=support_length
                    is_owned=when_is_owner
                    edit_mode_enabled=support_edit_mode
                    max_length=MAX_SUPPORT_LENGTH
                    errors=support_err
                    on_save=support_save_fn.clone()
                    node_ref=support_input
                    spawner
                    />
                <TextEditor
                    class="max-w-[20rem]"
                    id_prefix="aboutme"
                    title="Aboutme"
                    text=aboutme_text
                    text_length=aboutme_length
                    is_owned=when_is_owner
                    edit_mode_enabled=aboutme_edit_mode
                    max_length=MAX_ABOUTME_LENGTH
                    errors=aboutme_err
                    on_save=aboutme_save_fn.clone()
                    node_ref=aboutme_input
                    spawner
                    />
            </div>
        </Show>

        <Show when=when_is_closed>
            <div class="place-items-center hidden sm:grid sm:col-start-2 sm:row-start-1 grid-rows-[auto_auto_auto_auto_1fr] gap-4 px-4 py-2 bg-base01 rounded-lg">
                <Btn class="self-start bg-base02 " size=BtnSize::Small on_click=toggle_open><SVGArrowDown stroke=2.5 class="size-6 -rotate-90 "/></Btn>
                <p class="self-start text-[1rem] rounded-full size-[2.5rem] shrink-0 bg-base05"></p>
                <a href=link_tab_gallery class=class_side_minimized_gallery>"G"</a>
                <a href=link_tab_paws class=class_side_minimized_paws>"P"</a>
            </div>
        </Show>
    }
}

// <Btn class="self-start bg-base02 "><SVGArrowDown stroke=2.5 class="size-6 rotate-90 "/></Btn>
