use std::fmt::Debug;

use crate::Errs;
use crate::hook::Spawner;
use catsquad_client::{Client, Response, Sender, XMLSender};
use catsquad_log::prelude::*;
use catsquad_shared::{InviteAddErr, link_relative_reg_check, validate_email};
use leptos::attr::Xmlns;
use leptos::{html, prelude::*};
use leptos_router::NavigateOptions;
use leptos_router::hooks::{query_signal, use_navigate, use_query, use_query_map};
use leptos_router::params::{Params, ParamsError};
use web_sys::{HtmlInputElement, SubmitEvent};

use crate::PageState;
use crate::page::create_client;

// #[derive(
//     Debug,
//     Default,
//     Clone,
//     PartialEq,
//     PartialOrd,
//     strum::EnumString,
//     strum::Display,
//     strum::EnumIter,
//     strum::EnumIs,
// )]
// #[strum(serialize_all = "lowercase")]
// pub enum RegStage {
//     #[default]
//     None,
//     CheckEmail,
//     Reg,
// }

// #[derive(
//     Debug,
//     Default,
//     Clone,
//     PartialEq,
//     PartialOrd,
//     strum::EnumString,
//     strum::Display,
//     strum::EnumIter,
//     strum::EnumIs,
// )]
// #[strum(serialize_all = "lowercase")]
// pub enum RegQueryFields {
//     #[default]
//     None,
//     ErrGeneral,
//     ErrUsername,
//     ErrToken,
//     ErrPassword,
//     Stage,
//     Token,
//     Email,
// }

// #[derive(Clone, Copy)]
// pub struct Register {
//     pub err_general: RwSignal<String>,
//     pub err_username: RwSignal<String>,
//     pub err_token: RwSignal<String>,
//     pub err_password: RwSignal<String>,
//     // pub stage: RwQuery<RegStage>,
//     // pub email: RwQuery<String>,
//     // pub token: RwQuery<String>,
//     pub token_decoded: LocalResource<String>,
//     pub get_stage: StoredValue<Box<dyn Fn() -> RegStage + Sync + Send + 'static>>,
//     pub get_email: StoredValue<Box<dyn Fn() -> String + Sync + Send + 'static>>,
//     pub get_token: StoredValue<Box<dyn Fn() -> String + Sync + Send + 'static>>,

//     pub on_reg: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
//     pub on_invite: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
// }

#[derive(Clone, Copy)]
pub struct InviteState {
    // pub client: Client<TSender>,
    pub err_general: RwSignal<String>,
}

impl InviteState {
    pub fn new() -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
        }
    }

    pub async fn run_invite<TSender>(&self, client: &Client<TSender>, email: impl Into<String>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let email = email.into();
        let email = email.trim();
        let result = validate_email(email);
        match result {
            Ok(_) => {
                self.err_general.update(|v| v.clear());
            }
            Err(err) => {
                self.err_general.set(err);
                return;
            }
        }
        let result = client.invite_add(email).await.send().await.into_res().await;
        let _invite_res = match result {
            Ok(v) => v,
            Err(err) => {
                self.err_general.set(err.to_string());
                return;
            }
        };

        // invite_res.
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_invite_state() {
    init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;
    let client = &server.client;

    let invite = InviteState::new();
    invite.run_invite(client, "hello").await;
    assert!(!invite.err_general.get_untracked().is_empty());
    invite.run_invite(client, "").await;
    assert!(!invite.err_general.get_untracked().is_empty());
    invite.run_invite(client, "prime@heyadora.com").await;
    assert!(invite.err_general.get_untracked().is_empty());
}

#[component]
pub fn InviteForm() -> impl IntoView {
    let invite = InviteState::new();
    let spawner = Spawner::new();
    let navigator = use_navigate();
    let input_email: NodeRef<html::Input> = NodeRef::new();
    let on_invite = move |e: SubmitEvent| {
        e.prevent_default();
        let Some(email) = input_email
            .get_untracked()
            .map(|v: HtmlInputElement| v.value())
        else {
            return;
        };
        let navigator = navigator.clone();
        spawner.spawn(async move {
            let client = Client::new(XMLSender::new());
            invite.run_invite(&client, &email).await;
            if !invite.err_general.with_untracked(|v| v.is_empty()) {
                return;
            }
            let link = link_relative_reg_check(&email);
            navigator(&link, NavigateOptions::default());
        });
        //
    };
    view! {
        <form method="POST" action="" on:submit=on_invite class="flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full">
            <h1 class="text-[1.5rem]  text-center my-[4rem]">"REGISTRATION"</h1>
            <Errs class=move||"" error=move||invite.err_general.get()/>
            <div class="flex flex-col gap-0">
                <label for="email_invite" class="text-[1.2rem] ">"Email"</label>
                <input placeholder="alice@mail.com" id="email_invite" node_ref=input_email type="text" class="border-b-2 border-base05 w-full mt-1 " />
            </div>
            <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                <input type="submit" value="Register" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
            </div>
        </form>
    }
}

// pub fn use_register(
//     input_username: NodeRef<html::Input>,
//     input_email: NodeRef<html::Input>,
//     input_password: NodeRef<html::Input>,
//     input_password_confirmatoin: NodeRef<html::Input>,
// ) -> Register {
//     let page = PageState::get();

//     let navigate = leptos_router::hooks::use_navigate();

//     let err_general = RwSignal::<String>::new(RegQueryFields::ErrGeneral.to_string());
//     let err_username = RwSignal::<String>::new(RegQueryFields::ErrUsername.to_string());
//     let err_token = RwSignal::<String>::new(RegQueryFields::ErrToken.to_string());
//     let err_password = RwSignal::<String>::new(RegQueryFields::ErrPassword.to_string());
//     // let stage = RwQuery::<RegStage>::new(RegQueryFields::Stage.to_string());
//     // let token = RwQuery::<String>::new(RegQueryFields::Token.to_string());
//     // let email = RwQuery::<String>::new(RegQueryFields::Email.to_string());

//     let token_decoded = LocalResource::new(move || async move {
//         let token = token.get_or_default();
//         if token.is_empty() {
//             return String::new();
//         }
//         let result = api.decode_invite(token).send_native().await;

//         match result {
//             Ok(ServerRes::InviteToken {
//                 email,
//                 created_at,
//                 exp,
//             }) => email,
//             Ok(res) => {
//                 format!("error, expected InviteToken, received: {res:?}")
//             }
//             Err(err) => err.to_string(),
//         }
//     });

//     let on_invite = {
//         let navigate = navigate.clone();
//         move |e: SubmitEvent| {
//             e.prevent_default();
//             let navigate = navigate.clone();

//             let Some(email_field) = input_email.get_untracked() else {
//                 return;
//             };

//             let email_value = email_field.value();
//             let email_value = match proccess_email(&email_value) {
//                 Ok(email) => {
//                     err_general.clear();
//                     Some(email)
//                 }
//                 Err(err) => {
//                     error!("on_invite email \"{email_value}\" error: {err}");
//                     err_general.set(err);
//                     None
//                 }
//             };

//             let Some(email_value) = email_value else {
//                 return;
//             };
//             let email_value_clone = email_value.clone();

//             api.send_email_invite(email_value_clone)
//                 .send_web(move |result| {
//                     let email = email_value.clone();
//                     let navigate = navigate.clone();

//                     async move {
//                         match result {
//                             Ok(ServerRes::Ok) => {
//                                 // let result = api.profile().send_native().await;
//                                 // invite_completed.set(email.clone());
//                                 navigate(
//                                     &path::link_reg_check_email(email),
//                                     NavigateOptions {
//                                         ..Default::default()
//                                     },
//                                 );
//                             }
//                             Ok(res) => {
//                                 error!("expected Ok, received {res:?}");
//                                 err_general.set(format!("expected Ok, received {res:?}"));
//                             }

//                             Err(err) => {
//                                 error!("get invite err: {err}");
//                                 err_general.set(err.to_string());
//                             }
//                         }
//                     }
//                 });
//         }
//     };

//     let on_register = move |e: SubmitEvent| {
//         e.prevent_default();
//         let (Some(username), Some(password), Some(password_confirmation)) = (
//             input_username.get_untracked(),
//             // register_email.get(),
//             input_password.get_untracked(),
//             input_password_confirmatoin.get_untracked(),
//         ) else {
//             return;
//         };

//         let username_value = username.value();
//         let username_value = match proccess_username(username_value) {
//             Ok(v) => {
//                 err_username.clear();
//                 Some(v)
//             }
//             Err(err) => {
//                 let err = format!("on_register username input error: {err}");
//                 error!(err);
//                 err_username.set(err);
//                 None
//             }
//         };

//         let password_value = password.value();
//         let password_confirmation_value = password_confirmation.value();
//         let password_value =
//             match proccess_password(password_value, Some(password_confirmation_value)) {
//                 Ok(v) => {
//                     err_password.clear();
//                     Some(v)
//                 }
//                 Err(err) => {
//                     error!("on_register password input error: {err}");
//                     err_password.set(err);
//                     None
//                 }
//             };

//         if !token.is_some_untracked() {
//             err_general.set(String::from("token is missing from; invalid link"));
//             return;
//         } else {
//             err_general.clear();
//         }

//         let (Some(username), Some(password)) = (username_value, password_value) else {
//             return;
//         };

//         api.register(username, token.get_or_default_untracked(), password)
//             .send_web(move |result| {
//                 // let navigate = navigate.clone();
//                 async move {
//                     let err: Result<(), String> = match result {
//                         Ok(ServerRes::Ok) => {
//                             let res = page.update_auth_now().await;
//                             match res {
//                                 Ok(ServerRes::User { username }) => {
//                                     let result = page.update_auth_now().await;
//                                     match result {
//                                         Ok(S) => Ok(()),
//                                         Err(err) => Err(err.to_string()),
//                                     }
//                                 }
//                                 res => Err(format!("expected User, received {res:?}")),
//                             }
//                         }
//                         Ok(res) => Err(format!("error, expected OK, received: {res:?}")),
//                         Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenExpired)) => {
//                             Err("This invite link is already expired.".to_string())
//                         }
//                         Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenUsed)) => {
//                             Err("This invite link was already used.".to_string())
//                         }
//                         Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenNotFound)) => {
//                             Err("This invite link is invalid.".to_string())
//                         }
//                         Err(err) => Err(err.to_string()),
//                     };
//                     if let Err(err) = err {
//                         error!(err);
//                         err_general.set(err);
//                     }
//                 }
//             });
//     };

//     Register {
//         err_general,
//         err_username,
//         err_token,
//         err_password,
//         email,
//         stage,
//         token,
//         token_decoded,
//         on_invite: StoredValue::new(Box::new(on_invite)),
//         on_reg: StoredValue::new(Box::new(on_register)),
//     }
// }

// pub fn build_query_getter<QueryInput, MapFnOutput, MapFn>(
//     query: Memo<Result<QueryInput, ParamsError>>,
//     f: MapFn,
// ) -> impl Fn() -> MapFnOutput
// where
//     QueryInput: Params + Sync + Send + Clone + 'static,
//     MapFnOutput: Sync + Send + Default + 'static,
//     MapFn: Fn(&QueryInput) -> Option<MapFnOutput> + Clone,
// {
//     let fn_get_token = move || {
//         let f = f.clone();
//         query.with(|v| v.as_ref().ok().and_then(f).unwrap_or_default())
//     };

//     fn_get_token
// }
