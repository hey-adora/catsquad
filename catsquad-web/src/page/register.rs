use std::str::FromStr;

use crate::{Errs, Nav, hook::Spawner};
use catsquad_shared::{RegisterPageParams, RegisterPageStage};
use catsquad_web_utils::prelude::*;
use leptos::{html, prelude::*};

mod component_invite;
mod component_reg;
use component_invite::InviteForm;
use component_reg::RegisterForm;

#[component]
pub fn Register() -> impl IntoView {
    // let api = ApiWeb::new();
    let main_ref = NodeRef::new();
    let register_spawner = Spawner::new();
    let stage = RwQuery::<String>::new(RegisterPageParams::Stage.to_string());
    let stage = move || {
        stage
            .get()
            .and_then(|v| RegisterPageStage::from_str(&v).ok())
            .unwrap_or_default()
    };
    let email = RwQuery::<String>::new(RegisterPageParams::Email.to_string());

    view! {
        <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh]">
            <Nav/>
            <div class=move || format!("grid  text-base05 {}", if register_spawner.is_busy.get() {"items-center"} else {"justify-stretch"})>
                <Show when=move || register_spawner.is_busy.get()  >
                    <div class=move||"mx-auto text-[1.5rem]">
                        <h1>"LOADING..."</h1>
                    </div>
                </Show>
                <Show when=move || !register_spawner.is_busy.get() && stage().is_check_email()>
                    <div class="mx-auto flex flex-col gap-4 text-center">
                        <h1 class="text-[1.5rem] mt-[4rem]">"Verify Email"</h1>
                        <p class="max-w-[25rem]">"Verification email was sent to \""{ move || email.get() }"\" click the confirmtion link in the email."</p>
                    </div>
                </Show>
                <Show when=move|| !register_spawner.is_busy.get() && stage().is_invite()>
                    <InviteForm/>
                </Show>
                <Show when=move|| !register_spawner.is_busy.get() && stage().is_register()>
                    <RegisterForm/>
                </Show>
            </div>
        </main>
    }
}
