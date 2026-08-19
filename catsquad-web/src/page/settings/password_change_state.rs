use std::fmt::Debug;

use catsquad_client::{Client, Response, Sender};
use catsquad_shared::{
    PasswordChangeRes, PasswordChangeUpdateConfirmErr, PasswordChangeUpdateConfirmRes,
    validate_password,
};
use leptos::prelude::*;

pub struct PasswordChangeState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub err_general: RwSignal<String>,
    pub err_password: RwSignal<String>,
    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Clone for PasswordChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            err_general: self.err_general.clone(),
            err_password: self.err_general.clone(),
        }
    }
}

impl<TSender> Copy for PasswordChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

impl<TSender> PasswordChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            err_password: RwSignal::new(String::new()),
            client: StoredValue::new_local(client),
        }
    }

    pub async fn add(&self, email: impl Into<String>) -> Option<PasswordChangeRes> {
        self.err_general.update(|v| v.clear());
        self.err_password.update(|v| v.clear());

        let client = self.client.get_value();
        let result = client
            .password_change_add(email)
            .send()
            .await
            .into_json()
            .await;
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

    pub async fn confirm(
        &self,
        password_change_key: impl Into<String>,
        new_password: impl Into<String>,
        new_confirm: impl Into<String>,
    ) -> Option<PasswordChangeUpdateConfirmRes> {
        self.err_general.update(|v| v.clear());
        self.err_password.update(|v| v.clear());

        let new_password = new_password.into();
        let new_confirm = new_confirm.into();

        if new_confirm != new_password {
            self.err_password.set("passwords dont match".to_string());
            return None;
        }

        if let Err(res) = validate_password(&new_password) {
            self.err_password.set(res);
            return None;
        }

        let client = self.client.get_value();
        let result = client
            .password_change_update_confirm(password_change_key, new_password)
            .send()
            .await
            .into_json()
            .await;
        match result {
            Ok(v) => {
                return Some(v);
            }
            Err(PasswordChangeUpdateConfirmErr::NewPasswordInvalid(err)) => {
                self.err_password.set(err.to_string());
                return None;
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
async fn test_passowrd_change_state() {
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

    let password_change = PasswordChangeState::new(server.client.clone());

    {
        let res = password_change.add("invalid").await.unwrap();
        assert_eq!(password_change.err_general.get_untracked(), "");
        assert_eq!(password_change.err_password.get_untracked(), "");
    }

    let res = password_change.add("prime1@heyadora.com").await.unwrap();
    assert_eq!(password_change.err_general.get_untracked(), "");
    assert_eq!(password_change.err_password.get_untracked(), "");

    let key = server.password_change_get_latest_key().await;

    let res = password_change
        .confirm(key, "A6prime1@heyadora.com", "A6prime1@heyadora.com")
        .await
        .unwrap();
    assert_eq!(password_change.err_general.get_untracked(), "");
    assert_eq!(password_change.err_password.get_untracked(), "");

    let _result = server
        .session_add("prime1@heyadora.com", "A6prime1@heyadora.com")
        .await
        .unwrap();
}
