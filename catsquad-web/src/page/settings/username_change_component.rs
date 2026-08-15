use super::username_change_state::UsernameChangeState;
use crate::{
    BtnPrimary, ErrGeneral, Errs, LinkSecondary, PageState, hook::Spawner, page::create_client,
};
use catsquad_shared::{LINK_WEB_INDEX, link_relative_settings};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use web_sys::HtmlInputElement;

#[component]
pub fn UsernameChange() -> impl IntoView {
    let page = PageState::get();
    let link_back = move || link_relative_settings();
    let current_username = move || page.acc_username();
    let spawner = Spawner::new();
    let username_change = UsernameChangeState::new(create_client());
    let input_username = NodeRef::new();
    let input_password = NodeRef::new();
    let navigate = use_navigate();

    let on_confirm = move |_| {
        let (Some(new_username), Some(current_password)) = (
            input_username
                .get_untracked()
                .map(|v: HtmlInputElement| v.value()),
            input_password
                .get_untracked()
                .map(|v: HtmlInputElement| v.value()),
        ) else {
            return;
        };
        let navigate = navigate.clone();
        spawner.spawn(async move {
            let Some(result) = username_change.change(new_username, current_password).await else {
                return;
            };
            page.acc_username_set(result.username);
            navigate(link_back(), NavigateOptions::default());
        });
    };

    let general_errs = move || username_change.err_general.get();
    let username_errs = move || username_change.err_username.get();
    let is_loading = move || spawner.is_busy.get();

    view! {
        <div class=" bg-base01/80 absolute left-0 top-0 w-[100dvw] h-[100dvh] grid place-content-center">
            <a class="z-[1] absolute left-0 top-0 w-full h-full" href=link_back></a>
            <div class="z-[2] flex flex-col gap-6 shadow-lg bg-base00 rounded-lg px-6 py-4">
                <p class="text-[1.5rem] text-base0A text-center">"Username Change"</p>
                <ErrGeneral error=general_errs/>
                <div class="flex flex-col gap-2">
                    <label for="new_username">"New Username"</label>
                    <Errs error=username_errs/>
                    <input node_ref=input_username name="new_username" id="new_username" class="rounded-xl bg-base01 px-2 py-1 text-base0B" value=current_username />
                </div>
                <div class="flex flex-col gap-2">
                    <label for="current_password">"Current Password"</label>
                    <input node_ref=input_password type="password" name="current_password" id="current_password" class="rounded-xl bg-base01 px-2 py-1 text-base0B" />
                </div>
                <div class="ml-auto flex gap-2 ">
                    <BtnPrimary is_loading on_click=on_confirm>"Confirm"</BtnPrimary>
                    <LinkSecondary link=link_back>"Cancel"</LinkSecondary>
                </div>
            </div>
        </div>
    }
}
