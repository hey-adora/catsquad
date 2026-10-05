use crate::BtnPrimary;
use crate::Errs;
use crate::hook::Spawner;
use catsquad_client::{Client, Response, Sender, XMLSender};
use catsquad_shared::{link_relative_reg_check, validate_email};
use leptos::{html, prelude::*};
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use std::fmt::Debug;
use web_sys::{HtmlInputElement, MouseEvent};

#[derive(Clone, Copy)]
pub struct InviteState {
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
        let result = client.invite_add(email).send().await.into_json().await;
        let _invite_res = match result {
            Ok(v) => v,
            Err(err) => {
                self.err_general.set(err.to_string());
                return;
            }
        };
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_invite_state() {
    use catsquad_log::prelude::*;

    init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_invite_state").await;
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
    let on_invite = move |e: MouseEvent| {
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
            // TODO maybe dont put sensitive info in url? not sure
            let link = link_relative_reg_check(&email);
            navigator(&link, NavigateOptions::default());
        });
        //
    };
    view! {
        <div class="gap-2 flex flex-col px-[4rem] max-w-[25rem] mx-auto w-full">
            <h1 class="text-[1.5rem]  text-center mt-[4rem]">"Invite"</h1>
            <div class="flex flex-col gap-0">
                <label for="email_invite" class="text-[1.2rem] ">"Email"</label>
                <Errs error=invite.err_general />
                <input placeholder="alice@example.com" id="email_invite" node_ref=input_email type="text" class="rounded-xl bg-base01 px-2 py-1 text-base05" />
            </div>
            <div class="flex flex-col gap-[1.3rem] mx-auto my-4 text-center">
                <BtnPrimary on_click=on_invite >"Send"</BtnPrimary>
            </div>
        </div>
    }
}
