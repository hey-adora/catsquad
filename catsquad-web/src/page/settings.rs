use std::str::FromStr;

use crate::{BtnPrimary, BtnSecondary, LinkSecondary, Nav, PageState};
use catsquad_shared::{
    SettingsPageParams, SettingsPageStage, link_relative_settings_email_change_current_add,
    link_relative_settings_username_change,
};
use catsquad_web_utils::prelude::*;
use leptos::prelude::*;

mod email_change_state;
mod username_change_component;
mod username_change_state;

use username_change_component::UsernameChange;

#[component]
pub fn Settings() -> impl IntoView {
    let page = PageState::get();
    let username = move || page.acc_username();
    let email = move || page.acc_email();

    let stage = RwQuery::<String>::new(SettingsPageParams::Stage.to_string());
    let stage = move || {
        stage
            .get()
            .and_then(|v| SettingsPageStage::from_str(&v).ok())
            .unwrap_or_default()
    };

    let link_username_change = move || link_relative_settings_username_change();
    let when_stage_username_change = move || stage() == SettingsPageStage::UsernameChange;

    view! {
        <main class="relative font-hi text-base05 grid grid-rows-[auto_1fr] gap-4">
            <Nav/>
            <Show when=when_stage_username_change>
                <UsernameChange/>
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
                                    <LinkSecondary link=link_username_change>"Edit"</LinkSecondary>
                                </div>
                                <input class="rounded-xl bg-base01 px-2 py-1 text-base0B" value=username />
                            </div>
                            <div class="flex flex-col gap-2">
                                <div class="flex justify-between place-items-center">
                                    <p class="text-[1.1rem]">"Email"</p>
                                    <BtnSecondary>"Edit"</BtnSecondary>
                                </div>
                                <input type="email" class="rounded-xl bg-base01 px-2 py-1 text-base0B" value=email/>
                            </div>
                            <div class="flex flex-col gap-2">
                                <div class="flex justify-between place-items-center">
                                    <p class="text-[1.1rem]">"Password"</p>
                                    <BtnSecondary>"Edit"</BtnSecondary>
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
