use crate::{
    BtnPrimary, ErrGeneral, Errs, LinkSecondary, PageState,
    hook::Spawner,
    page::{create_client, settings::PasswordChangeState},
};
use catsquad_shared::{
    LINK_WEB_INDEX, PasswordResetStage, link_relative_login,
    link_relative_login_password_reset_check, link_relative_login_password_reset_finished,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use web_sys::HtmlInputElement;

#[component]
pub fn PasswordReset(
    #[prop(optional, into)] password_reset_stage_tracked: Option<Callback<(), PasswordResetStage>>,
    #[prop(optional, into)] password_reset_stage_untracked: Option<
        Callback<(), PasswordResetStage>,
    >,
    #[prop(optional, into)] email_tracked: Option<Callback<(), String>>,
    #[prop(optional, into)] password_reset_key_untracked: Option<Callback<(), String>>,
) -> impl IntoView {
    let link_back = move || link_relative_login();

    let spawner = Spawner::new();
    let password_reset = PasswordChangeState::new(create_client());

    let email = move || email_tracked.map(|v| v.run(())).unwrap_or_default();
    let password_reset_key = move || {
        password_reset_key_untracked
            .map(|v| v.run(()))
            .unwrap_or_default()
    };
    let password_reset_stage_tracked = move || {
        password_reset_stage_tracked
            .map(|v| v.run(()))
            .unwrap_or_default()
    };
    let password_reset_stage_untracked = move || {
        password_reset_stage_untracked
            .map(|v| v.run(()))
            .unwrap_or_default()
    };

    let navigate = use_navigate();

    let input_email = NodeRef::new();
    let input_new_password = NodeRef::new();
    let input_new_password_confirm = NodeRef::new();

    let view_msg = move || {
        match password_reset_stage_tracked() {
            PasswordResetStage::Add => view! {
                <div id="pss_reset_confirm_component" class="flex flex-col gap-2 " >
                    <div  class="flex flex-col gap-2 ">
                        <label for="user_email" class="">"Email"</label>
                        <input name="user_email" id="user_email" node_ref=input_email type="email" class="rounded-xl bg-base01 px-2 py-1 text-base0B" />
                    </div>
                </div>
            }.into_any(),
            PasswordResetStage::Check => view! {
                <p id="pss_reset_check_component">"Confirmation was sent to \""
                <span class="text-base0E">{email}</span>
                "\""</p>
            }.into_any(),
            PasswordResetStage::Confirm => view! {
                <div id="pss_reset_confirm_component" class="flex flex-col gap-2 " >

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
            PasswordResetStage::Finished => view! {
                <p id="pss_reset_finished_component">"Password was reset for "{email}". Try logging in again."</p>
            }.into_any(),
      }
    };
    let view_text = move || match password_reset_stage_tracked() {
        PasswordResetStage::Add => "Send",
        PasswordResetStage::Check => "",
        PasswordResetStage::Confirm => "Confirm",
        PasswordResetStage::Finished => "",
    };
    let on_confirm = move |_| {
        let navigate = navigate.clone();
        match password_reset_stage_untracked() {
            PasswordResetStage::Add => {
                let Some(user_email) = input_email.get_untracked().map(|v| v.value()) else {
                    return;
                };
                spawner.spawn(async move {
                    let Some(_result) = password_reset.add(&user_email).await else {
                        return;
                    };
                    let link = link_relative_login_password_reset_check(user_email);
                    navigate(&link, NavigateOptions::default());
                });
            }
            PasswordResetStage::Check => {
                //
            }
            PasswordResetStage::Confirm => {
                let (Some(new_password), Some(new_password_confirmation)) = (
                    input_new_password.get_untracked().map(|v| v.value()),
                    input_new_password_confirm
                        .get_untracked()
                        .map(|v| v.value()),
                ) else {
                    return;
                };
                let password_reset_key = password_reset_key();
                spawner.spawn(async move {
                    let Some(result) = password_reset
                        .confirm(password_reset_key, new_password, new_password_confirmation)
                        .await
                    else {
                        return;
                    };
                    let link = link_relative_login_password_reset_finished(result.email);
                    navigate(&link, NavigateOptions::default());
                });
            }
            PasswordResetStage::Finished => {
                //
            }
        }
    };
    let general_errs = move || password_reset.err_general.get();
    let is_loading = move || spawner.is_busy.get();
    let when_confirm_btn = move || match password_reset_stage_tracked() {
        PasswordResetStage::Add => true,
        PasswordResetStage::Check => false,
        PasswordResetStage::Confirm => true,
        PasswordResetStage::Finished => false,
    };

    view! {
        <div id="password_reset_component" class=" bg-base01/80 absolute left-0 top-0 w-[100dvw] h-[100dvh] grid place-content-center">
            <a class="z-[1] absolute left-0 top-0 w-full h-full" href=link_back></a>
            <div class="z-[2] flex flex-col gap-6 shadow-lg bg-base00 rounded-lg px-6 py-4">
                <p class="text-[1.5rem] text-base0A text-center">"Password Reset"</p>
                <ErrGeneral id=move||"passowrd_reset_general_error" error=general_errs/>
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
