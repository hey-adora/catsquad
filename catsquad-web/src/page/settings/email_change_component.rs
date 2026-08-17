use super::email_change_state::EmailChangeState;
use super::username_change_state::UsernameChangeState;
use crate::{
    BtnDelete, BtnPrimary, ErrGeneral, Errs, LinkSecondary, PageState, hook::Spawner,
    page::create_client,
};
use catsquad_shared::{
    EmailCangeStage, LINK_WEB_INDEX, link_relative_settings,
    link_relative_settings_email_change_canceled,
    link_relative_settings_email_change_current_check_email,
    link_relative_settings_email_change_finish, link_relative_settings_email_change_finished,
    link_relative_settings_email_change_new_add,
    link_relative_settings_email_change_new_check_email,
    link_relative_settings_email_change_new_confirm,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use web_sys::{HtmlInputElement, MouseEvent};

#[component]
pub fn EmailChange(
    #[prop(optional, into)] email_change_stage_tracked: Option<Callback<(), EmailCangeStage>>,
    #[prop(optional, into)] email_change_stage_untracked: Option<Callback<(), EmailCangeStage>>,
    #[prop(optional, into)] email_change_key_untracked: Option<Callback<(), String>>,
    #[prop(optional, into)] token_untracked: Option<Callback<(), String>>,
    #[prop(optional, into)] new_email_tracked: Option<Callback<(), String>>,
) -> impl IntoView {
    let page = PageState::get();
    let link_back = move || link_relative_settings();
    let email_change_stage_tracked = move || {
        email_change_stage_tracked
            .map(|v| v.run(()))
            .unwrap_or_default()
    };
    let email_change_stage_untracked = move || {
        email_change_stage_untracked
            .map(|v| v.run(()))
            .unwrap_or_default()
    };
    let email_change_key_untracked = move || {
        email_change_key_untracked
            .map(|v| v.run(()))
            .unwrap_or_default()
    };
    let token_untracked = move || token_untracked.map(|v| v.run(())).unwrap_or_default();
    let new_email_tracked = move || new_email_tracked.map(|v| v.run(())).unwrap_or_default();

    // let when_stage_email_current_add =
    //     move || email_change_stage() == EmailCangeStage::ChangeEmailCurrentAdd;
    // let when_stage_email_current_check_email =
    //     move || email_change_stage() == EmailCangeStage::ChangeEmailCurrentCheckEmail;
    // let when_stage_email_current_confirm =
    //     move || email_change_stage() == EmailCangeStage::ChangeEmailCurrentConfirm;
    // let when_stage_email_new_add =
    //     move || email_change_stage() == EmailCangeStage::ChangeEmailNewAdd;
    // let when_stage_email_new_check_email =
    //     move || email_change_stage() == EmailCangeStage::ChangeEmailNewCheckEmail;

    let navigate = use_navigate();
    let spawner = Spawner::new();
    let email_change = EmailChangeState::new(create_client());
    let input_new_email = NodeRef::new();

    let success_msg = RwSignal::new(String::new());
    let when_success = move || success_msg.with(|v| !v.is_empty());

    let link_back = move || link_relative_settings();
    let is_loading = move || spawner.is_busy.get();
    let general_errs = move || email_change.err_general.get();

    let on_click = {
        let navigate = navigate.clone();

        move |_: MouseEvent| {
            let navigate = navigate.clone();

            spawner.spawn(async move {
                match email_change_stage_untracked() {
                    EmailCangeStage::ChangeEmailCurrentAdd => {
                        let Some(result) = email_change.current_add().await else {
                            return;
                        };
                        let key = result.key.clone();
                        let link = link_relative_settings_email_change_current_check_email(key);
                        navigate(&link, NavigateOptions::default());
                    }
                    EmailCangeStage::ChangeEmailCurrentConfirm => {
                        let email_change_key = email_change_key_untracked();
                        let token = token_untracked();
                        let Some(_) = email_change
                            .current_confirm(email_change_key.clone(), token)
                            .await
                        else {
                            return;
                        };
                        let link = link_relative_settings_email_change_new_add(email_change_key);
                        navigate(&link, NavigateOptions::default());
                    }
                    EmailCangeStage::ChangeEmailNewAdd => {
                        let Some(new_email) = input_new_email
                            .try_get_untracked()
                            .flatten()
                            .map(|v: HtmlInputElement| v.value())
                        else {
                            return;
                        };
                        let email_change_key = email_change_key_untracked();
                        let Some(_) = email_change
                            .new_add(email_change_key.clone(), new_email.clone())
                            .await
                        else {
                            return;
                        };
                        let link = link_relative_settings_email_change_new_check_email(
                            &email_change_key,
                            new_email,
                        );
                        navigate(&link, NavigateOptions::default());
                    }
                    EmailCangeStage::ChangeEmailNewConfirm => {
                        let email_change_key = email_change_key_untracked();
                        let token = token_untracked();
                        let Some(_) = email_change
                            .new_confirm(email_change_key.clone(), token)
                            .await
                        else {
                            return;
                        };
                        let link = link_relative_settings_email_change_finish(email_change_key);
                        navigate(&link, NavigateOptions::default());
                    }
                    EmailCangeStage::ChangeEmailFinish => {
                        let email_change_key = email_change_key_untracked();
                        let Some(new) = email_change
                            .finish(email_change_key.clone())
                            .await
                            .and_then(|v| v.new)
                        else {
                            return;
                        };
                        page.acc_email_set(new.email);
                        let link = link_relative_settings_email_change_finished(email_change_key);
                        navigate(&link, NavigateOptions::default());
                    }
                    EmailCangeStage::ChangeEmailCurrentCheckEmail
                    | EmailCangeStage::ChangeEmailNewCheckEmail => {
                        let email_change_key = email_change_key_untracked();
                        let Some(_) = email_change.resend(email_change_key).await else {
                            success_msg.update(|v| v.clear());
                            return;
                        };
                        success_msg.set("Confirmation email was re-sent.".to_string());
                    }
                    EmailCangeStage::ChangeEmailFinished => (),
                    EmailCangeStage::ChangeEmailCanceled => (),
                };
            });
        }
    };

    let on_cancel = move |_| {
        success_msg.update(|v| v.clear());
        let email_change_key = email_change_key_untracked();
        let navigate = navigate.clone();
        spawner.spawn(async move {
            let Some(_) = email_change.cancel(email_change_key).await else {
                success_msg.update(|v| v.clear());
                return;
            };
            let link = link_relative_settings_email_change_canceled();
            navigate(&link, NavigateOptions::default());
        });
    };

    let when_cancel = move || match email_change_stage_tracked() {
        EmailCangeStage::ChangeEmailCurrentAdd
        | EmailCangeStage::ChangeEmailFinished
        | EmailCangeStage::ChangeEmailCanceled => false,
        _ => true,
    };

    let when_primary = move || match email_change_stage_tracked() {
        EmailCangeStage::ChangeEmailCanceled | EmailCangeStage::ChangeEmailFinished => false,
        _ => true,
    };

    let view_msg = move || {
        match email_change_stage_tracked() {
        EmailCangeStage::ChangeEmailCurrentAdd => view! {
            <p id="current_add_component" class="text-center">
                "Send confirmation email to "
                <span class="text-base0E">{move || page.acc_email()}</span>
                "."
            </p>
        }
        .into_any(),
        EmailCangeStage::ChangeEmailCurrentCheckEmail => view! {
            <p id="current_check_component" class="text-center">
                "Email Confirmation was sent to your email "
                <span class="text-base0E">{move || page.acc_email()}</span>
                ", confirm it to continue."
            </p>
        }
        .into_any(),
        EmailCangeStage::ChangeEmailCurrentConfirm => view! {
            <p id="current_confirm_component" class="text-center">"Confirm to continue."</p>
        }
        .into_any(),
        EmailCangeStage::ChangeEmailNewAdd => view! {
            <div id="new_add_component" class="flex flex-col gap-2 ">
                <label for="new_email" class="">"New Email"</label>
                <input name="new_email" id="new_email" node_ref=input_new_email type="email" placeholder="alice@example.com" class="rounded-xl bg-base01 px-2 py-1 text-base0B" />
            </div>
        }
        .into_any(),
        EmailCangeStage::ChangeEmailNewCheckEmail => view! {
            <p id="new_check_component" class="text-center">
                "Email Confirmation was sent to "
                <span class="text-base0E">{new_email_tracked}</span>
                ", confirm it to continue."
            </p>
        }
        .into_any(),
        EmailCangeStage::ChangeEmailNewConfirm => view! {
            <p id="new_confirm_component" class="text-center">"Confirm to continue."</p>
        }
        .into_any(),
        EmailCangeStage::ChangeEmailFinish => view! {
            <p id="finish_component" class="text-center">"final confirm."</p>
        }
        .into_any(),
        EmailCangeStage::ChangeEmailFinished => view! {
            <p id="finished_component" class="text-center">"completed."</p>
        }
        .into_any(),
        EmailCangeStage::ChangeEmailCanceled => view! {
            <p id="canceled_component" class="text-center">"canceled."</p>
        }
        .into_any(),
    }
    };

    let primary_btn_text = move || match email_change_stage_tracked() {
        EmailCangeStage::ChangeEmailCurrentAdd => "Send",
        EmailCangeStage::ChangeEmailCurrentCheckEmail => "Resend",
        EmailCangeStage::ChangeEmailCurrentConfirm => "Confirm",
        EmailCangeStage::ChangeEmailNewAdd => "Send",
        EmailCangeStage::ChangeEmailNewCheckEmail => "Resend",
        EmailCangeStage::ChangeEmailNewConfirm => "Confirm",
        EmailCangeStage::ChangeEmailFinish => "Confirm",
        EmailCangeStage::ChangeEmailFinished => "",
        EmailCangeStage::ChangeEmailCanceled => "",
    };

    view! {
        <div class=" bg-base01/80 absolute left-0 top-0 w-[100dvw] h-[100dvh] flex place-items-center justify-center">
            <a class="z-[1] absolute left-0 top-0 w-full h-full" href=link_back></a>
            <div class="z-[2] max-w-[25rem] w-full flex flex-col gap-6 shadow-lg bg-base00 rounded-lg px-6 py-4">
                <p class="text-[1.5rem] text-base0A text-center">"Email Change"</p>
                <Show when=when_success>
                    <p class="text-base0B text-center">{ move || success_msg.get() }</p>
                </Show>
                <ErrGeneral id=move||"err_general" error=general_errs />
                {view_msg}
                <div class="flex flex-wrap justify-end gap-2 ">
                    <Show when=when_cancel>
                        <BtnDelete id=move||"cancel_btn" class=move||"mr-auto" is_loading on_click=on_cancel.clone()>"Cancel"</BtnDelete>
                    </Show>
                    <Show when=when_primary>
                        <BtnPrimary id=move||"confirm_btn" class=move||"" is_loading on_click=on_click.clone() >{primary_btn_text}</BtnPrimary>
                    </Show>
                    <LinkSecondary id=move||"close_btn" link=link_back>"Close"</LinkSecondary>
                </div>

            </div>
        </div>
    }
}
