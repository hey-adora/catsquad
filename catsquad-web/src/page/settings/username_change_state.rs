use std::fmt::Debug;
use std::time::Duration;

use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{LINK_WEB_SETTINGS, UserUpdateUsernameErr, UserUpdateUsernameRes};
use catsquad_web_utils::prelude::*;
use leptos::html;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::params::Params;
use leptos_router::{NavigateOptions, hooks::use_query};
use web_sys::{HtmlInputElement, SubmitEvent};

pub struct UsernameChangeState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub err_general: RwSignal<String>,
    pub err_username: RwSignal<String>,
    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Clone for UsernameChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            err_general: self.err_general.clone(),
            err_username: self.err_general.clone(),
        }
    }
}

impl<TSender> Copy for UsernameChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

impl<TSender> UsernameChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            err_username: RwSignal::new(String::new()),
            client: StoredValue::new_local(client),
        }
    }

    pub async fn change(
        &self,
        new_username: impl Into<String>,
        current_password: impl Into<String>,
    ) -> Option<UserUpdateUsernameRes> {
        self.err_general.update(|v| v.clear());
        self.err_username.update(|v| v.clear());

        let client = self.client.get_value();
        let result = client
            .user_update_username(current_password, new_username)
            .send()
            .await
            .into_json()
            .await;
        match result {
            Ok(v) => {
                return Some(v);
            }
            Err(UserUpdateUsernameErr::InvalidUsername(err)) => {
                self.err_username.set(err.to_string());
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
async fn test_username_change_state() {
    use catsquad_api::{auth::create_auth_cookie_str, utils::rng_str};
    use catsquad_shared::{MAX_USERNAME_LENGTH, PostState};
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;

    let (_user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    server
        .inject_header(header::COOKIE, create_auth_cookie_str(session1.clone()))
        .await;

    let username_change = UsernameChangeState::new(server.client.clone());
    assert!(username_change.err_general.get_untracked().is_empty());

    let result = username_change.change("prime2", "invalid").await;
    assert_eq!(result, None);
    assert!(!username_change.err_general.get_untracked().is_empty());

    let invalid_name = rng_str(MAX_USERNAME_LENGTH + 1);
    let result = username_change
        .change(invalid_name, "235j4t49ngerigrog#IOTNOnfo")
        .await;
    assert_eq!(result, None);
    assert!(username_change.err_general.get_untracked().is_empty());
    assert!(!username_change.err_username.get_untracked().is_empty());

    let result = username_change
        .change("prime2", "235j4t49ngerigrog#IOTNOnfo")
        .await;
    assert!(result.is_some());
    assert!(username_change.err_general.get_untracked().is_empty());
    assert!(username_change.err_username.get_untracked().is_empty());
}
