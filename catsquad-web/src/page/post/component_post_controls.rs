use crate::{
    BtnDelete, BtnPrimary, BtnSecondary, PageState,
    hook::Spawner,
    page::{create_client, post::post_api::PostApi},
};
use catsquad_shared::{PostState, RoleValue, UserState};
use leptos::prelude::*;

#[component]
pub fn PostControls(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let page = PageState::get();
    let post_state = post_api.post_state;
    let delete_post = move |_| {
        let post_id = post_id.get();
        if post_id == 0 {
            return;
        }
        spawner.spawn(async move {
            let client = create_client();
            post_api.delete(&client, post_id).await;
        });
    };

    let state = move || match post_state.get() {
        Some(PostState::Draft) => "Draft",
        Some(PostState::Active) => "Public",
        Some(PostState::Hidden) => "Hidden",
        None => "",
    };

    let user_state = move || match post_api.author_state.get() {
        UserState::Active => "Active",
        UserState::Suspended => "Suspended",
    };

    let user_state_color = move || match post_api.author_state.get() {
        UserState::Active => "bg-base0B text-base01",
        UserState::Suspended => "bg-base08 text-base01",
    };

    let state_color = move || match post_state.get() {
        Some(PostState::Draft) => "bg-base03",
        Some(PostState::Active) => "bg-base0B text-base01",
        Some(PostState::Hidden) => "bg-base08 text-base01",
        None => "",
    };
    let when_active = move || match post_state.get() {
        Some(PostState::Active) => true,
        _ => false,
    };
    let when_hidden = move || match post_state.get() {
        Some(PostState::Hidden) => true,
        _ => false,
    };

    let on_state_change = move |_e| {
        spawner.spawn(async move {
            let client = create_client();
            post_api
                .toggle_state(&client, post_id.get_untracked())
                .await;
        });
    };

    let class_state = move || format!("{} font-bold px-2 py-1 rounded", state_color());
    let class_user_state = move || format!("{} font-bold px-2 py-1 rounded", user_state_color());

    let is_my_post = move || page.acc_username() == post_api.author_username.get();
    let not_my_post = move || !is_my_post();
    let when_can_see_controls =
        move || is_my_post() || page.acc_contains_perm(RoleValue::PermissionSeePostControls);
    let is_loading = move || spawner.is_busy.get();

    let control_title = move || {
        if is_my_post() {
            "Control Panel"
        } else {
            "Moderation Panel"
        }
    };

    // TODO add modal to on_change_state
    view! {
        <Show when=when_can_see_controls>
            <div class="col-span-2 flex flex-col flex-wrap gap-2 ">
                <p class="text-base0A mx-4 ">{control_title}</p>
                <div class="flex flex-wrap gap-2 rounded-md border-2 border-base0A mx-2 px-2 md:px-6 py-2 ">
                    <Show when=is_my_post>
                        <p class=class_state>{state}</p>
                    </Show>
                    <Show when=not_my_post>
                        <p class=class_state>"Post "{state}</p>
                        <p class=class_user_state>"User "{user_state}</p>
                    </Show>

                    <Show when=is_my_post>

                        <Show when=when_hidden>
                            <BtnPrimary
                                modal_enable=true
                                modal_title="publish post"
                                modal_text="are you sure you want to publish the post?"
                                is_loading
                                on_click=on_state_change
                                class="sm:ml-auto"
                            >
                                "Publish"
                            </BtnPrimary>
                        </Show>
                        <Show when=when_active>
                            <BtnSecondary
                                modal_enable=true
                                modal_title="hide post"
                                modal_text="are you sure you want to hide the post?"
                                is_loading
                                on_click=on_state_change
                                class="sm:ml-auto"
                            >
                                "Hide"
                            </BtnSecondary>
                        </Show>
                        <BtnDelete
                            modal_enable=true
                            modal_title="delete post"
                            modal_text="are you sure you want to delete the post?"
                            on_click=delete_post
                        >
                            "Delete"
                        </BtnDelete>

                    </Show>

                </div>
            </div>
        </Show>
    }
}
// <SVGTrash class="size-[1rem] text-base08 "/>
