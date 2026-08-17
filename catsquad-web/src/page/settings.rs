use std::str::FromStr;

use crate::{BtnPrimary, BtnSecondary, LinkSecondary, Nav, PageState};
use catsquad_shared::{
    EmailCangeStage, SettingsPageParams, SettingsPageStage,
    link_relative_settings_email_change_current_add, link_relative_settings_password_change,
    link_relative_settings_username_change,
};
use catsquad_web_utils::prelude::*;
use leptos::prelude::*;

mod email_change_component;
mod email_change_state;
mod password_change_component;
mod password_change_state;
mod username_change_component;
mod username_change_state;

use email_change_component::EmailChange;
use username_change_component::UsernameChange;

#[component]
pub fn Settings() -> impl IntoView {
    let page = PageState::get();
    let username = move || page.acc_username();
    let email = move || page.acc_email();

    let stage = RwQuery::<SettingsPageStage>::new(SettingsPageParams::Stage.to_string());
    let stage = move || stage.get_or_default();
    // let stage = move || {
    //     stage
    //         .get()
    //         .and_then(|v| SettingsPageStage::from_str(&v).ok())
    //         .unwrap_or_default()
    // };
    let email_stage =
        RwQuery::<EmailCangeStage>::new(SettingsPageParams::EmailChangeStage.to_string());
    let email_stage_tracked = move || email_stage.get_or_default();
    let email_stage_untracked = move || email_stage.get_untracked().unwrap_or_default();

    let email_change_key = RwQuery::<String>::new(SettingsPageParams::EmailChangeKey.to_string());
    let email_change_key_untracked = move || email_change_key.get_untracked().unwrap_or_default();

    let token = RwQuery::<String>::new(SettingsPageParams::Token.to_string());
    let token_untracked = move || token.get_untracked().unwrap_or_default();

    let new_email = RwQuery::<String>::new(SettingsPageParams::NewEmail.to_string());
    let new_email_tracked = move || new_email.get().unwrap_or_default();
    // let email_stage = move || {
    //     email_stage
    //         .get()
    //         .and_then(|v| SettingsPageStage::from_str(&v).ok())
    //         .unwrap_or_default()
    // };

    let link_username_change = move || link_relative_settings_username_change();
    let link_email_change = move || link_relative_settings_email_change_current_add();
    let link_password_change = move || link_relative_settings_password_change();
    let when_stage_username_change = move || stage() == SettingsPageStage::UsernameChange;
    let when_stage_email_change = move || stage() == SettingsPageStage::EmailChange;

    view! {
        <main id="settings_page" class="relative font-hi text-base05 grid grid-rows-[auto_1fr] gap-4">
            <Nav/>
            <Show when=when_stage_username_change>
                <UsernameChange/>
            </Show>
            <Show when=when_stage_email_change>
                <EmailChange
                    email_change_stage_tracked=email_stage_tracked
                    email_change_stage_untracked=email_stage_untracked
                    email_change_key_untracked
                    token_untracked
                    new_email_tracked
                    />
            </Show>
            <div class="px-[2rem] mx-auto max-w-[30rem] w-full">
                <h1 class="text-[1.5rem] text-base0A font-bold mb-[2rem]">"Settings"</h1>
                <div class="grid grid-cols-[1fr] gap-2  ">
                    <div class="flex flex-col gap-2">
                        <p class="text-[1.3rem] text-base0A">"User"</p>
                        <div class="flex flex-col gap-4">
                            <div class="flex flex-col gap-2">
                                <div class="flex justify-between place-items-center">
                                    <p class="text-[1.1rem]">"Username"</p>
                                    <LinkSecondary id=move||"username_change_btn" link=link_username_change>"Edit"</LinkSecondary>
                                </div>
                                <input id="current_user_username" class="rounded-xl bg-base01 px-2 py-1 text-base0B" value=username />
                            </div>
                            <div class="flex flex-col gap-2">
                                <div class="flex justify-between place-items-center">
                                    <p class="text-[1.1rem]">"Email"</p>
                                    <LinkSecondary id=move||"email_change_btn" link=link_email_change>"Edit"</LinkSecondary>
                                </div>
                                <input id="current_user_email" type="email" class="rounded-xl bg-base01 px-2 py-1 text-base0B" value=email/>
                            </div>
                            <div class="flex flex-col gap-2">
                                <div class="flex justify-between place-items-center">
                                    <p class="text-[1.1rem]">"Password"</p>
                                    <LinkSecondary id=move||"password_change_btn" link=link_password_change>"Edit"</LinkSecondary>
                                </div>
                                <input type="password" class="rounded-xl bg-base01 px-2 py-1 text-base0B" value="***********"/>
                            </div>
                        </div>
                    </div>
                </div>

            </div>
        </main>

    }
}

// <div class="grid place-items-center">
//     <div class="bg-base03 rounded-full size-[6rem]"></div>
// </div>
