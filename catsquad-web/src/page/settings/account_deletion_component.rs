use super::account_deletion_state::AccountDeletionState;
use crate::{BtnPrimary, LinkSecondary, PageState, hook::Spawner, page::create_client};
use catsquad_log::prelude::*;
use catsquad_shared::{
    AccountDeletionStage, Uuid, link_relative_settings,
    link_relative_settings_account_deletion_check_email,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};

#[component]
pub fn AccountDeletion(
    #[prop(optional, into)] account_deletion_stage: Signal<AccountDeletionStage>,
    #[prop(optional, into)] account_deletion_token: Signal<Uuid>,
) -> impl IntoView {
    let page = PageState::get();
    let spawner = Spawner::new();
    let account_deletion = AccountDeletionState::new(create_client());
    let navigate = use_navigate();

    let link_back = move || link_relative_settings();
    let is_loading = move || spawner.is_busy.get();
    let user_email = move || page.acc_email();

    let on_confirm = move |_| {
        let navigate = navigate.clone();
        match account_deletion_stage.get() {
            AccountDeletionStage::Add => {
                spawner.spawn(async move {
                    let Some(_result) = account_deletion.send_confirmation().await else {
                        return;
                    };
                    let link = link_relative_settings_account_deletion_check_email();
                    navigate(&link, NavigateOptions::default());
                });
            }
            AccountDeletionStage::Email => {
                //
            }
            AccountDeletionStage::Confirm => {
                let token = account_deletion_token.get_untracked();
                spawner.spawn(async move {
                    let Some(_result) = account_deletion.confirm(token).await else {
                        return;
                    };
                    page.logout().await;
                });
            }
        }
    };

    let view_msg = move || match account_deletion_stage.get() {
        AccountDeletionStage::Add => view! {
            <p>"Send account deletion confirmation to your email?"</p>
        }
        .into_any(),
        AccountDeletionStage::Email => view! {
            <p id="pss_add_component">"Account deletion confirmation was sent to \""
            <span class="text-base0E">{user_email}</span>
            "\""</p>
        }
        .into_any(),
        AccountDeletionStage::Confirm => view! {
            <p>"Send account deletion confirmation to your email?"</p>
        }
        .into_any(),
    };

    let when_confirm_btn = move || match account_deletion_stage.get() {
        AccountDeletionStage::Add => true,
        AccountDeletionStage::Email => false,
        AccountDeletionStage::Confirm => true,
    };

    view! {
        <div id="username_change_component" class=" bg-base01/80 absolute left-0 top-0 w-[100dvw] h-[100dvh] grid place-content-center">
            <a class="z-[1] absolute left-0 top-0 w-full h-full" href=link_back></a>
            <div class="z-[2] flex flex-col gap-6 shadow-lg bg-base00 rounded-lg px-6 py-4">
                <p class="text-[1.5rem] text-base0A text-center">"Account Deletion"</p>
                {view_msg}
                <div class="ml-auto flex gap-2 ">
                    <Show when=when_confirm_btn>
                        <BtnPrimary id="confirm_btn" is_loading on_click=on_confirm.clone()>"Confirm"</BtnPrimary>
                    </Show>
                    <LinkSecondary id="cancel_btn" link=link_back>"Cancel"</LinkSecondary>
                </div>
            </div>
        </div>
    }
}
// <ErrGeneral id=move||"username_change_general_error" error=general_errs/>
// <div class="flex flex-col gap-2">
//     <label for="new_username">"New Username"</label>
//     <Errs error=username_errs/>
//     <input node_ref=input_username name="new_username" id="new_username" class="rounded-xl bg-base01 px-2 py-1 text-base0B" value=current_username />
// </div>
// <div class="flex flex-col gap-2">
//     <label for="current_password">"Current Password"</label>
//     <input node_ref=input_password type="password" name="current_password" id="current_password" class="rounded-xl bg-base01 px-2 py-1 text-base0B" />
// </div>
// <div class="ml-auto flex gap-2 ">
//     <BtnPrimary id=move||"confirm_btn" is_loading on_click=on_confirm>"Confirm"</BtnPrimary>
//     <LinkSecondary id=move||"cancel_btn" link=link_back>"Cancel"</LinkSecondary>
// </div>
