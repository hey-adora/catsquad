use std::fmt::Debug;
use std::time::Duration;

use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{EmailChangeRes, LINK_WEB_SETTINGS};
use catsquad_web_utils::prelude::*;
use leptos::html;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::params::Params;
use leptos_router::{NavigateOptions, hooks::use_query};
use web_sys::{HtmlInputElement, SubmitEvent};

pub struct EmailChangeState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub err_general: RwSignal<String>,
    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Clone for EmailChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            // token: self.token.clone(),
            // email_change_key: self.email_change_key.clone(),
            err_general: self.err_general.clone(),
            // state: self.state.clone(),
        }
    }
}

impl<TSender> Copy for EmailChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

impl<TSender> EmailChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            // state: RwSignal::new(FormState::Loading),
            // token: RwSignal::new(String::new()),
            // email_change_key: RwSignal::new(email_change_token.into()),
            err_general: RwSignal::new(String::new()),
            client: StoredValue::new_local(client),
        }
    }

    pub async fn current_add(&self) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let result = client.email_change_add().send().await.into_json().await;
        self.handle_result(result)
    }

    pub async fn current_confirm(
        &self,
        email_change_key: impl Into<String>,
        token: impl Into<String>,
    ) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let token = token.into();
        let result = client
            .email_change_update_current_confirm(email_change_key, token)
            .send()
            .await
            .into_json()
            .await;
        self.handle_result(result)
    }

    pub async fn new_add(
        &self,
        email_change_key: impl Into<String>,
        new_email: impl Into<String>,
    ) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let new_email = new_email.into();
        let result = client
            .email_change_update_new_add(email_change_key, new_email)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub async fn new_confirm(
        &self,
        email_change_key: impl Into<String>,
        token: impl Into<String>,
    ) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let result = client
            .email_change_update_new_confirm(email_change_key, token)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub async fn finish(&self, email_change_key: impl Into<String>) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let result = client
            .email_change_update_finish(email_change_key)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub async fn resend(&self, email_change_key: impl Into<String>) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();

        let result = client
            .email_change_resend(email_change_key)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub async fn cancel(&self, email_change_key: impl Into<String>) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();

        let result = client
            .email_change_update_cancel(email_change_key)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub fn handle_result<E: ToString>(
        &self,
        result: Result<EmailChangeRes, E>,
    ) -> Option<EmailChangeRes> {
        match result {
            Ok(v) => {
                return Some(v);
            }
            Err(err) => {
                self.err_general.set(err.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_state() {
    use catsquad_api::auth::create_auth_cookie_str;
    use catsquad_shared::PostState;
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;

    let (user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    server
        .inject_header(header::COOKIE, create_auth_cookie_str(session1.clone()))
        .await;

    let email_change = EmailChangeState::new(server.client.clone());

    let email_change_res = email_change.current_add().await.unwrap();
    let current_token = server
        .email_change_get_current_token(0, &user1, email_change_res.key.clone())
        .await;
    assert!(email_change.err_general.get_untracked().is_empty());

    {
        let result = email_change
            .current_confirm(email_change_res.key.clone(), "invalid")
            .await;
        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    let email_change_res = email_change
        .current_confirm(email_change_res.key.clone(), current_token.clone())
        .await
        .unwrap();
    assert!(email_change.err_general.get_untracked().is_empty());

    {
        let result = email_change
            .current_confirm(email_change_res.key.clone(), current_token)
            .await;
        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    {
        let result = email_change
            .new_add(email_change_res.key.clone(), "prime2")
            .await;
        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    let email_change_res = email_change
        .new_add(email_change_res.key.clone(), "prime2@heyadora.com")
        .await
        .unwrap();

    let new_token = server
        .email_change_get_new_token(0, &user1, email_change_res.key.clone())
        .await;

    {
        let result = email_change
            .new_confirm(email_change_res.key.clone(), "invalid")
            .await;

        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    let result = email_change
        .new_confirm(email_change_res.key.clone(), new_token)
        .await
        .unwrap();

    let email_change_res = email_change
        .finish(email_change_res.key.clone())
        .await
        .unwrap();

    let result = server.user_get_by_session_key(&session1).await.unwrap();
    assert_eq!(result.email, "prime2@heyadora.com");
}
