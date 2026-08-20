use crate::{BtnPrimary, Display, Errs, Nav, PageState, hook::Spawner, page::create_client};
use catsquad_client as api;
use catsquad_log::prelude::*;
use catsquad_shared::{
    LoginPageParams, LoginPageStage, PasswordResetStage, SensitiveUserRes,
    link_relative_invite_get_by_key, link_relative_login_password_reset_send,
    link_relative_register,
};
use catsquad_web_utils::prelude::RwQuery;
use leptos::{html, prelude::*};
use web_sys::HtmlInputElement;

mod login_state;
mod password_reset_component;

// fn hello()  {
//     let a = 10;
//     let b = 10;
//     let c = a + b;

// }

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
    let email_untracked = move || email.get_untracked().unwrap_or_default();

    let password_reset_key = RwQuery::<String>::new(LoginPageParams::Token.to_string());
    let password_reset_key = move || password_reset_key.get_untracked().unwrap_or_default();

    let on_login = move |e: web_sys::SubmitEvent| {
        e.prevent_default();

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
        // spawn_l
        //
    };

    let link_password_reset = link_relative_login_password_reset_send();

    view! {
        <main id="login_page" class="grid grid-rows-[auto_1fr] h-screen">
            <Nav/>
            <div class=move || format!("grid  text-base05 {}", if login_spawner.is_busy.get() {"items-center"} else {"justify-stretch"})>
                <Show when=move||login_spawner.is_busy.get()>
                    <h1>"LOADING..."</h1>
                </Show>
                <Display when=move||!login_spawner.is_busy.get() class=move||"">
                    <form method="POST" action="" on:submit=on_login class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full")>
                        <h1 class="text-[1.5rem]  text-center my-[4rem]">"LOGIN"</h1>
                        <div class=move||format!("text-red-600 {}", if err_general.with(|v| v.is_empty()) {"hidden"} else {""})>{move || { err_general.get() }}</div>
                        <div class="flex flex-col justify-center gap-[3rem]">
                            <div class="flex flex-col gap-0">
                                <label for="email" class="text-[1.2rem] ">"Email"</label>
                                <input placeholder="alice@mail.com" id="email" node_ref=input_email type="email" class="border-b-2 border-base05" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password" class="text-[1.2rem] ">"Password"</label>
                                <input id="password" node_ref=input_password type="password" class="border-b-2 border-base05" />
                            </div>
                            <a href=link_password_reset class="underline">"forgot password?"</a>
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input id="login_btn" type="submit" value="Login" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
                            <a href=link_relative_register() class="underline">"or Register"</a>
                        </div>
                    </form>
                </Display>
            </div>
        </main>
    }
}
// <Errs error=|| email_err.get()/>
// // <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if email_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
// //     <ul class="list-disc ml-[1rem]">
// //         {move || email_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
// //     </ul>
// // </div>

// use leptos::html;
// use leptos::{html::Input, prelude::*};

// use crate::api::{Api, ApiWeb, ServerLoginErr, ServerRes};
// use crate::path::{
//     link_login, link_login_form_password_send, link_reg_invite, link_settings, query_form_password,
// };
// use crate::view::app::components::nav::Nav;
// use crate::view::app::hook::use_password_change::{
//     ChangePasswordBtnStage, ChangePasswordFormStage, use_password_change,
// };
// // use crate::view::app::{Acc, GlobalState};
// use crate::view::toolbox::prelude::*;
// // use tracing::{error, trace};
// use web_sys::SubmitEvent;

// #[component]
// pub fn Page() -> impl IntoView {
//     let global_state = PageState::get();
//     let main_ref: NodeRef<html::Main> = NodeRef::new();
//     let input_email: NodeRef<html::Input> = NodeRef::new();
//     let input_password: NodeRef<html::Input> = NodeRef::new();
//     let general_err = RwSignal::new(String::new());
//     let email_err = RwSignal::new(String::new());
//     let navigate = leptos_router::hooks::use_navigate();
//     // let api = ApiWeb::new();
//     // let api_reset_password = ApiWeb::new();

//     let change_password_email = NodeRef::new();
//     let change_password_password = NodeRef::new();
//     let change_password_password_confirmation = NodeRef::new();
//     let change_password = use_password_change(
//         api_reset_password,
//         change_password_email,
//         change_password_password,
//         change_password_password_confirmation,
//     );

//     let on_login = move |e: SubmitEvent| {
//         e.prevent_default();
//         let (Some(email), Some(password)) = (input_email.get(), input_password.get()) else {
//             return;
//         };

//         let email = email.value();
//         let password = password.value();
//         general_err.set(String::new());

//         trace!("login dispatched");
//         api.login(email, password)
//             .send_web(move |result| async move {
//                 match result {
//                     Ok(ServerRes::Ok) => {
//                         global_state.update_auth();
//                     }
//                     Ok(res) => {
//                         error!("expected Ok, received {res:?}");
//                     }
//                     Err(err) => {
//                         let r = general_err.try_set(err.to_string());
//                         if r.is_some() {
//                             error!("global state acc was disposed somehow");
//                         }
//                     }
//                 }
//             });
//     };

//     let view_current_stage_label = move |current_stage: u8, view_stage: u8| {
//         let (text, style) = if current_stage == view_stage {
//             ("Current", "text-base0C")
//         } else if current_stage > view_stage {
//             ("Done", "text-base0B")
//         } else {
//             ("Next", "text-base03")
//         };

//         view! {
//             <span class=style>"["{text}"] "</span>
//         }
//     };

//     let view_current_password_change_stage_label = move |stage: ChangePasswordFormStage| {
//         view_current_stage_label(
//             change_password.form_stage.get_or_default() as u8,
//             stage as u8,
//         )
//     };

//     view! {
//         <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh] relative">
//             <Nav/>
//             <div class=move || format!("grid  text-base05 {}", if api.is_pending_tracked() {"items-center"} else {"justify-stretch"})>
//                 <div class=move||format!("mx-auto text-[1.5rem] {}", if api.is_pending_tracked() {""} else {"hidden"})>
//                     <h1>"LOADING..."</h1>
//                 </div>
//                 <form method="POST" action="" on:submit=on_login class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if api.is_pending_tracked() || api.is_succ_tracked()  {"hidden"} else {""})>
//                     <h1 class="text-[1.5rem]  text-center my-[4rem]">"LOGIN"</h1>
//                     <div class=move||format!("text-red-600 {}", if general_err.with(|v| v.is_empty()) {"hidden"} else {""})>{move || { general_err.get() }}</div>
//                     <div class="flex flex-col justify-center gap-[3rem]">
//                         <div class="flex flex-col gap-0">
//                             <label for="email" class="text-[1.2rem] ">"Email"</label>
//                             <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if email_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
//                                 <ul class="list-disc ml-[1rem]">
//                                     {move || email_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
//                                 </ul>
//                             </div>
//                             <input placeholder="alice@mail.com" id="email" node_ref=input_email type="email" class="border-b-2 border-base05" />
//                         </div>
//                         <div class="flex flex-col gap-0">
//                             <label for="password" class="text-[1.2rem] ">"Password"</label>
//                             <input id="password" node_ref=input_password type="password" class="border-b-2 border-base05" />
//                         </div>
//                         <a href=link_login_form_password_send() class="underline">"forgot password?"</a>
//                     </div>
//                     <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
//                         <input id="login_btn" type="submit" value="Login" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
//                         <a href=link_reg_invite() class="underline">"or Register"</a>
//                     </div>
//                 </form>
//             </div>

//             <div class=move || format!("absolute top-0 left-0 w-full h-full grid place-items-center bg-base00/80 {}", if !change_password.form_stage.get_or_default().is_none() { "flex" } else { "hidden" } )>
//                 <div class="flex flex-col px-[2rem] md:px-[4rem] max-w-[30rem] mx-auto w-full border-0 border-base05 bg-base01">
//                     <h2 class="text-[1.5rem]  text-center mt-[4rem] mb-[1rem]">"Reset Password"</h2>
//                     <div class=move||format!("text-red-600 text-center  {}", if change_password.err_general.is_some() { "visible" } else { "hidden" } )>{move || { change_password.err_general.get() }}</div>
//                     <div class="flex flex-col gap-6 mt-[1rem]">
//                         <ol class="text-[1.2rem] list-decimal grid gap-2">
//                             <li>
//                                 { move || view_current_password_change_stage_label(ChangePasswordFormStage::Send)}
//                                 "Input the account email address "
//                                 <input node_ref=change_password_email placeholder="user@example.com"  class=move || format!("bg-base02 mt-2 pl-2 {}", if change_password.form_stage.get_or_default() == ChangePasswordFormStage::Send { "visible" } else {"hidden"} ) type="email" />
//                             </li>
//                             <li>
//                                 { move || view_current_password_change_stage_label(ChangePasswordFormStage::Check)}
//                                 "Click on the confirmation link that was sent to "<span class="text-base0E">{move || change_password.email.get_or_else("specified email.")}</span>"."
//                             </li>
//                             <li>
//                                 { move || view_current_password_change_stage_label(ChangePasswordFormStage::Confirm)}
//                                 "Input the new password. "
//                                 <div class=move || format!(" {}", if change_password.form_stage.get_or_default() == ChangePasswordFormStage::Confirm { "visible" } else {"hidden"} )>
//                                     <input node_ref=change_password_password placeholder="new password" class="bg-base02 mt-2 pl-2" type="password" />
//                                     <input node_ref=change_password_password_confirmation placeholder="new password" class="bg-base02 mt-2 pl-2" type="password" />
//                                 </div>
//                             </li>
//                             <li>
//                                 { move || view_current_password_change_stage_label(ChangePasswordFormStage::Finish)}
//                                 "Password changed successfully."
//                             </li>
//                         </ol>
//                     </div>

//                     <div class=move || format!("w-full flex gap-4 my-[4rem] justify-center {}", if api.is_pending_tracked() {"visible"} else {"hidden"})>
//                         "loading..."
//                     </div>
//                     <div class= move || format!("flex flex-row gap-[1.3rem] my-[4rem] justify-between {}", if api.is_pending_tracked() {"hidden"} else {"visible"})>
//                         <a href=link_login() class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950">"Cancel"</a>
//                         <form method="POST" on:submit=change_password.on_change.to_fn() action="" class=move || format!("flex flex-col {}", if change_password.btn_stage.run() != ChangePasswordBtnStage::None { "visible" } else { "hidden" }) >
//                             <input type="submit" value=move || if api.is_pending_tracked() { "Saving...".to_string() } else { change_password.btn_stage.run().to_string() } disabled=move || api.is_pending_tracked() class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
//                         </form>
//                     </div>

//                 </div>
//             </div>

//         </main>
//     }
// }
