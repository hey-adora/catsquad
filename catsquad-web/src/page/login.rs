use crate::{BtnPrimary, Display, Nav, PageState, hook::Spawner, page::create_client};
use catsquad_log::prelude::*;
use catsquad_shared::{
    LoginPageParams, LoginPageStage, PasswordResetStage, link_relative_login_password_reset_send,
    link_relative_register,
};
use catsquad_web_utils::prelude::RwQuery;
use leptos::prelude::*;
use web_sys::HtmlInputElement;

mod login_state;
mod password_reset_component;

use password_reset_component::PasswordReset;

#[component]
pub fn Login() -> impl IntoView {
    let page = PageState::get();
    let login_spawner = Spawner::new();

    // let login_spawner = Spawner::new();
    // let h =  c;
    let input_email = NodeRef::new();
    let input_password = NodeRef::new();
    let err_general = RwSignal::new(String::new());

    let page_stage = RwQuery::<LoginPageStage>::new(LoginPageParams::PageStage.to_string());
    let page_stage = move || page_stage.get_or_default();

    let password_stage =
        RwQuery::<PasswordResetStage>::new(LoginPageParams::PssResetStage.to_string());
    let password_stage_tracked = move || password_stage.get_or_default();
    let password_stage_untracked = move || password_stage.get_untracked().unwrap_or_default();

    let email = RwQuery::<String>::new(LoginPageParams::Email.to_string());
    let email_tracked = move || email.get().unwrap_or_default();
    let email_untracked = move || email.get_untracked().unwrap_or_default();

    let password_reset_key = RwQuery::<String>::new(LoginPageParams::Token.to_string());
    let password_reset_key_untracked =
        move || password_reset_key.get_untracked().unwrap_or_default();

    let on_login = move |e: web_sys::MouseEvent| {
        let (Some(email), Some((elm_password, password))) = (
            input_email.get().map(|v: HtmlInputElement| v.value()),
            input_password.get().map(|v: HtmlInputElement| {
                let val = v.value();
                (v, val)
            }),
        ) else {
            return;
        };
        elm_password.set_value("");
        let client = create_client();
        login_spawner.spawn(async move {
            let result = client
                .session_add(email, password)
                .send()
                .await
                .into_json()
                .await;
            match result {
                Ok(_user) => {
                    page.update_auth().await;
                }
                Err(err) => {
                    let r = err_general.try_set(err.to_string());
                    if r.is_some() {
                        error!("global state acc was disposed somehow");
                    }
                }
            }
        });
    };

    let link_password_reset = link_relative_login_password_reset_send();
    let when_password_reset = move || page_stage() == LoginPageStage::PssReset;

    view! {
        <main id="login_page" class="grid grid-rows-[auto_1fr] h-screen">
            <Nav/>
            <Show when=when_password_reset>
                <PasswordReset
                        password_reset_stage_tracked=password_stage_tracked
                        password_reset_stage_untracked=password_stage_untracked
                        password_reset_key_untracked=password_reset_key_untracked
                        email_tracked
                    />
            </Show>
            <div class=move || format!("grid  text-base05 {}", if login_spawner.is_busy.get() {"items-center"} else {"justify-stretch"})>
                <Show when=move||login_spawner.is_busy.get()>
                    <h1>"LOADING..."</h1>
                </Show>
                <Display when=move||!login_spawner.is_busy.get() class=move||"">
                    <div class="gap-2 flex flex-col px-[4rem] max-w-[25rem] mx-auto w-full">
                        <h1 class="text-[1.5rem]  text-center mt-[4rem]">"Login"</h1>
                        <div class=move||format!("text-red-600 {}", if err_general.with(|v| v.is_empty()) {"hidden"} else {""})>{move || { err_general.get() }}</div>
                        <div class="flex flex-col justify-center gap-2">
                            <div class="flex flex-col gap-0">
                                <label for="email" class="text-[1.2rem] ">"Email"</label>
                                <input placeholder="alice@mail.com" id="email" node_ref=input_email type="email" class="rounded-xl bg-base01 px-2 py-1 text-base05" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password" class="text-[1.2rem] ">"Password"</label>
                                <input id="password" node_ref=input_password type="password" class="rounded-xl bg-base01 px-2 py-1 text-base05" />
                            </div>
                            <a id="password_reset_link" href=link_password_reset class="underline">"forgot password?"</a>
                        </div>
                        <div class="flex flex-col gap-2 mx-auto my-4 text-center">
                            <BtnPrimary on_click=on_login >"Login"</BtnPrimary>
                            <a href=link_relative_register() class="underline">"or Register"</a>
                        </div>
                    </div>
                </Display>
            </div>
        </main>
    }
}
