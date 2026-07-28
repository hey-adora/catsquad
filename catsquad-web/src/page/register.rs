use std::str::FromStr;

use crate::{Errs, Nav, hook::Spawner};
use catsquad_shared::{RegisterPageParams, RegisterPageStage};
use catsquad_web_utils::prelude::*;
use leptos::{html, prelude::*};

mod component_invite;
mod component_reg;
use component_invite::InviteForm;
use component_reg::RegisterForm;

// use hook_register::use_register;

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

    // let register_username: NodeRef<html::Input> = NodeRef::new();
    // let register_email: NodeRef<html::Input> = NodeRef::new();
    // let register_password: NodeRef<html::Input> = NodeRef::new();
    // let register_password_confirmation: NodeRef<html::Input> = NodeRef::new();

    // let a = RwQuery::<String>::new("");
    // let a = a.fn_get;

    // let reg = use_register(
    //     // api,
    //     register_username,
    //     register_email,
    //     register_password,
    //     register_password_confirmation,
    // );

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
                    <div class=move||"mx-auto flex flex-col gap-2 text-center">
                        <h1 class="text-[1.5rem] my-[4rem]">"VERIFY EMAIL"</h1>
                        <p class="max-w-[30rem]">"Verification email was sent to \""{ move || email.get() }"\" click the confirmtion link in the email."</p>
                    </div>
                </Show>
                <Show when=move|| !register_spawner.is_busy.get() && stage().is_invite()>
                    <InviteForm/>
                    // <form method="POST" action="" on:submit=reg.on_invite.to_fn() class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if reg.stage.get_or_default().is_none() && !api.is_pending_tracked() {""} else {"hidden"})>
                    //     <h1 class="text-[1.5rem]  text-center my-[4rem]">"REGISTRATION"</h1>
                    //     <div class=move||format!("text-red-600 text-center {}", if reg.err_general.is_some() {""} else {"hidden"})>{move || { reg.err_general.get_or_default() }}</div>
                    //     <div class="flex flex-col gap-0">
                    //         <label for="email_invite" class="text-[1.2rem] ">"Email"</label>
                    //         <input placeholder="alice@mail.com" id="email_invite" node_ref=register_email type="text" class="border-b-2 border-base05 w-full mt-1 " />
                    //     </div>
                    //     <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                    //         <input type="submit" value="Register" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
                    //     </div>
                    // </form>
                </Show>
                <Show when=move|| !register_spawner.is_busy.get() && stage().is_register()>
                    <RegisterForm/>
                    //
                </Show>
            </div>
        </main>
    }
}
