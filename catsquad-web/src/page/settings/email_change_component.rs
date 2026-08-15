use super::email_change_state::EmailChangeState;
use super::username_change_state::UsernameChangeState;
use crate::{
    BtnPrimary, ErrGeneral, Errs, LinkSecondary, PageState, hook::Spawner, page::create_client,
};
use catsquad_shared::{
    EmailCangeStage, LINK_WEB_INDEX, link_relative_settings,
    link_relative_settings_email_change_current_check_email,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use web_sys::HtmlInputElement;

#[component]
pub fn EmailChange(
    #[prop(optional, into)] email_change_stage: Option<Callback<(), EmailCangeStage>>,
) -> impl IntoView {
    let page = PageState::get();
    let link_back = move || link_relative_settings();
    let email_change_stage = move || email_change_stage.map(|v| v.run(())).unwrap_or_default();
    // let current_username = move || page.acc_username();
    // let username_change = UsernameChangeState::new(create_client());
    // let input_username = NodeRef::new();
    // let input_password = NodeRef::new();
    // let navigate = use_navigate();

    // let general_errs = move || username_change.err_general.get();
    // let username_errs = move || username_change.err_username.get();
    // let is_loading = move || spawner.is_busy.get();

    // let email_change_add_text =
    //     move || format!("Send confirmation email to \"{}\"", page.acc_email());
    let when_stage_email_current_add =
        move || email_change_stage() == EmailCangeStage::ChangeEmailCurrentAdd;
    let when_stage_email_current_check_email =
        move || email_change_stage() == EmailCangeStage::ChangeEmailCurrentCheckEmail;

    view! {
        <div class=" bg-base01/80 absolute left-0 top-0 w-[100dvw] h-[100dvh] grid place-content-center">
            <a class="z-[1] absolute left-0 top-0 w-full h-full" href=link_back></a>
            <div class="z-[2] max-w-[25rem] w-full flex flex-col gap-6 shadow-lg bg-base00 rounded-lg px-6 py-4">
                <Show when=when_stage_email_current_add>
                    <EmailChangeCurrentAdd/>
                </Show>
                <Show when=when_stage_email_current_check_email>
                    <EmailChangeCurrentCheckEmail/>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn EmailChangeCurrentAdd() -> impl IntoView {
    let page = PageState::get();
    let link_back = move || link_relative_settings();
    let spawner = Spawner::new();
    let email_change_add_text = move || {
        view! {
            "Send confirmation email to "
            <span class="text-base0E">{move || page.acc_email()}</span>
            "."
        }
    };
    let navigate = use_navigate();
    let is_loading = move || spawner.is_busy.get();
    let email_change = EmailChangeState::new(create_client());
    let general_errs = move || email_change.err_general.get();

    let on_confirm = move |_| {
        let navigate = navigate.clone();
        spawner.spawn(async move {
            let Some(_) = email_change.current_add().await else {
                return;
            };
            let link = link_relative_settings_email_change_current_check_email();
            navigate(&link, NavigateOptions::default());
        });
    };

    view! {
        <p class="text-[1.5rem] text-base0A text-center">"Email Change"</p>
        <ErrGeneral error=general_errs />
        <p class="text-center">{email_change_add_text}</p>
        <div class="ml-auto flex gap-2 ">
            <BtnPrimary is_loading on_click=on_confirm >"Send"</BtnPrimary>
            <LinkSecondary link=link_back>"Cancel"</LinkSecondary>
        </div>
    }
}

#[component]
pub fn EmailChangeCurrentCheckEmail() -> impl IntoView {
    let page = PageState::get();
    let link_back = move || link_relative_settings();
    let email_change_current_check_email = move || {
        view! {
            "Email Confirmation was sent to your email "
            <span class="text-base0E">{move || page.acc_email()}</span>
            ", confirm it to continue."
        }
    };

    view! {
        <p class="text-[1.5rem] text-base0A text-center">"Email Change"</p>
        <p class="text-center">{email_change_current_check_email}</p>
        <div class="ml-auto flex gap-2 ">
            <LinkSecondary link=link_back>"Cancel"</LinkSecondary>
        </div>
    }
}
