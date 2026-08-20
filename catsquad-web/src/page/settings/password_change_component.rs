use super::password_change_state::PasswordChangeState;
use crate::{
    BtnPrimary, ErrGeneral, Errs, LinkSecondary, PageState, hook::Spawner, page::create_client,
};
use catsquad_shared::{
    EmailCangeStage, LINK_WEB_INDEX, PasswordCangeStage, link_relative_settings,
    link_relative_settings_password_change_check_email,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use web_sys::HtmlInputElement;

#[component]
pub fn PasswordChange(
    // #[prop(optional, into)] password_change_stage: Signal<PasswordCangeStage>,
    #[prop(optional, into)] password_change_stage_tracked: Option<Callback<(), PasswordCangeStage>>,
    #[prop(optional, into)] password_change_stage_untracked: Option<
        Callback<(), PasswordCangeStage>,
    >,
    #[prop(optional, into)] password_change_key: Option<Callback<(), String>>,
) -> impl IntoView {
    // let page = PageState::get();
    let link_back = move || link_relative_settings();
    // let current_username = move || page.acc_username();
    let spawner = Spawner::new();
    let password_change = PasswordChangeState::new(create_client());
    let password_change_key = move || password_change_key.map(|v| v.run(())).unwrap_or_default();
    let password_change_stage_tracked = move || {
        password_change_stage_tracked
            .map(|v| v.run(()))
            .unwrap_or_default()
    };
    let password_change_stage_untracked = move || {
        password_change_stage_untracked
            .map(|v| v.run(()))
            .unwrap_or_default()
    };

    // PasswordCangeStage::
    // EmailCangeStage
    // let input_username = NodeRef::new();
    // let input_password = NodeRef::new();
    let navigate = use_navigate();

    // let on_confirm = move |_| {
    //     let (Some(new_username), Some(current_password)) = (
    //         input_username
    //             .get_untracked()
    //             .map(|v: HtmlInputElement| v.value()),
    //         input_password
    //             .get_untracked()
    //             .map(|v: HtmlInputElement| v.value()),
    //     ) else {
    //         return;
    //     };
    //     let navigate = navigate.clone();
    //     spawner.spawn(async move {
    //         let Some(result) = username_change.change(new_username, current_password).await else {
    //             return;
    //         };
    //         page.acc_username_set(result.username);
    //         navigate(link_back(), NavigateOptions::default());
    //     });
    // };

    let page = PageState::get();
    let input_new_password = NodeRef::new();
    let input_new_password_confirm = NodeRef::new();
    let user_email = move || page.acc_email();
    let view_msg = move || {
        match password_change_stage_tracked() {
            PasswordCangeStage::PasswordChangeAdd => view! {
                <p id="pss_add_component">"Send confirmation to \""
                <span class="text-base0E">{user_email}</span>
                "\""</p>
            }.into_any(),
            PasswordCangeStage::PasswordChangeCheckEmail => view! {
                <p id="pss_check_component">"Confirmation was sent to \""
                <span class="text-base0E">{user_email}</span>
                "\""</p>
            }.into_any(),
            PasswordCangeStage::PasswordChangeConfirm => view! {
                <div id="pss_confirm_component" class="flex flex-col gap-2 " >

                    <div  class="flex flex-col gap-2 ">
                        <label for="new_password" class="">"New Password"</label>
                        <input name="new_password" id="new_password" node_ref=input_new_password type="password" class="rounded-xl bg-base01 px-2 py-1 text-base0B" />
                    </div>

                    <div  class="flex flex-col gap-2 ">
                        <label for="new_password_confirm" class="">"New Password Confirm"</label>
                        <input name="new_password_confirm" id="new_password_confirm" node_ref=input_new_password_confirm type="password" class="rounded-xl bg-base01 px-2 py-1 text-base0B" />
                    </div>

                </div>

            }.into_any(),
            // PasswordCangeStage::PasswordChangeFinished => view! {
            //     <p>""</p>
            // }.into_any(),
      }
    };
    let view_text = move || match password_change_stage_tracked() {
        PasswordCangeStage::PasswordChangeAdd => "Send",
        PasswordCangeStage::PasswordChangeCheckEmail => "",
        PasswordCangeStage::PasswordChangeConfirm => "Confirm",
        // PasswordCangeStage::PasswordChangeFinished => "",
    };
    let on_confirm = move |_| {
        let navigate = navigate.clone();
        match password_change_stage_untracked() {
            PasswordCangeStage::PasswordChangeAdd => {
                let user_email = user_email();
                spawner.spawn(async move {
                    let Some(_result) = password_change.add(user_email).await else {
                        return;
                    };
                    let link = link_relative_settings_password_change_check_email();
                    navigate(&link, NavigateOptions::default());
                });
            }
            PasswordCangeStage::PasswordChangeCheckEmail => {
                //
            }
            PasswordCangeStage::PasswordChangeConfirm => {
                let (Some(new_password), Some(new_password_confirmation)) = (
                    input_new_password.get_untracked().map(|v| v.value()),
                    input_new_password_confirm
                        .get_untracked()
                        .map(|v| v.value()),
                ) else {
                    return;
                };
                let password_change_key = password_change_key();
                spawner.spawn(async move {
                    let Some(_result) = password_change
                        .confirm(password_change_key, new_password, new_password_confirmation)
                        .await
                    else {
                        return;
                    };
                    page.logout().await;
                });
            }
        }
    };
    let general_errs = move || password_change.err_general.get();
    // let username_errs = move || username_change.err_username.get();
    let is_loading = move || spawner.is_busy.get();
    let when_confirm_btn = move || match password_change_stage_tracked() {
        PasswordCangeStage::PasswordChangeAdd => true,
        PasswordCangeStage::PasswordChangeCheckEmail => false,
        PasswordCangeStage::PasswordChangeConfirm => true,
        // PasswordCangeStage::PasswordChangeFinished => "",
    };

    view! {
        <div id="password_change_component" class=" bg-base01/80 absolute left-0 top-0 w-[100dvw] h-[100dvh] grid place-content-center">
            <a class="z-[1] absolute left-0 top-0 w-full h-full" href=link_back></a>
            <div class="z-[2] flex flex-col gap-6 shadow-lg bg-base00 rounded-lg px-6 py-4">
                <p class="text-[1.5rem] text-base0A text-center">"Password Change"</p>
                <ErrGeneral id=move||"passowrd_change_general_error" error=general_errs/>
                {view_msg}
                <div class="ml-auto flex gap-2 ">
                    <Show when=when_confirm_btn>
                        <BtnPrimary id=move||"confirm_btn" is_loading on_click=on_confirm.clone()>{view_text}</BtnPrimary>
                    </Show>
                    <LinkSecondary id=move||"close_btn" link=link_back>"Cancel"</LinkSecondary>
                </div>
            </div>
        </div>
    }
}
