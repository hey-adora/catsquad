use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::warn;
use catsquad_shared::{EmailChangeRes, Uuid};
use leptos::prelude::*;
use std::fmt::Debug;

pub struct EmailChangeState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    //ui
    pub err_general: RwSignal<String>,
    pub current_email: RwSignal<String>,
    pub new_email: RwSignal<String>,
    //params
    pub email_change_id: StoredValue<i64>,
    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Clone for EmailChangeState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    fn clone(&self) -> Self {
        Self {
            //
            err_general: self.err_general,
            current_email: self.current_email,
            new_email: self.new_email,
            //
            client: self.client,
            email_change_id: self.email_change_id,
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
            current_email: RwSignal::new(String::new()),
            new_email: RwSignal::new(String::new()),
            //
            email_change_id: StoredValue::new(0),
            client: StoredValue::new_local(client),
        }
    }

    pub fn get_params(&self) -> Option<(i64,)> {
        let email_change_id = self.email_change_id.get_value();
        if email_change_id == 0 {
            warn!("trying to run email change without initialization");
            return None;
        }
        Some((email_change_id,))
    }

    pub async fn init(&self, email_change_id: i64) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();

        let result = client
            .email_change_get_by_id(email_change_id)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {
                self.email_change_id.set_value(v.id);
                self.current_email.set(v.current.email.clone());
                self.new_email
                    .set(v.new.as_ref().map(|v| v.email.clone()).unwrap_or_default());
                Some(v)
            }
            Err(err) => {
                self.err_general.set(err.to_string());
                None
            }
        }
    }

    pub async fn current_add(&self) -> Option<EmailChangeRes> {
        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let result = client.email_change_add().send().await.into_json().await;
        match result {
            Ok(v) => {
                self.email_change_id.set_value(v.id);
                self.current_email.set(v.current.email.clone());
                Some(v)
            }
            Err(err) => {
                self.err_general.set(err.to_string());
                None
            }
        }
    }

    // pub fn is_busy(&self) -> bool {
    //     self.client.with_value(|v| v.is)
    // }

    pub async fn current_confirm(&self, token: Uuid) -> Option<()> {
        let (email_change_id,) = self.get_params()?;

        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let token = token.into();
        let result = client
            .email_change_update_current_confirm(email_change_id, token)
            .send()
            .await
            .into_json()
            .await;
        self.handle_result(result)
    }

    pub async fn new_add(&self, new_email: impl Into<String>) -> Option<EmailChangeRes> {
        let (email_change_id,) = self.get_params()?;

        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let new_email = new_email.into();
        let result = client
            .email_change_update_new_add(email_change_id, new_email)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub async fn new_confirm(&self, token: Uuid) -> Option<()> {
        let (email_change_id,) = self.get_params()?;

        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let result = client
            .email_change_update_new_confirm(email_change_id, token)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub async fn finish(&self) -> Option<()> {
        let (email_change_id,) = self.get_params()?;

        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();
        let result = client
            .email_change_update_finish(email_change_id)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub async fn resend(&self) -> Option<EmailChangeRes> {
        let (email_change_id,) = self.get_params()?;

        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();

        let result = client
            .email_change_resend(email_change_id)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub async fn cancel(&self) -> Option<()> {
        let (email_change_id,) = self.get_params()?;

        self.err_general.update(|v| v.clear());
        let client = self.client.get_value();

        let result = client
            .email_change_update_cancel(email_change_id)
            .send()
            .await
            .into_json()
            .await;

        self.handle_result(result)
    }

    pub fn handle_result<R, E: ToString>(&self, result: Result<R, E>) -> Option<R> {
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
    use catsquad_shared::uuid_to_str;
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_email_change_state").await;

    let (user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    server
        .inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session1)),
        )
        .await;

    let email_change = EmailChangeState::new(server.client.clone());

    let email_change_res = email_change.current_add().await.unwrap();
    let current_token = server
        .email_change_get_current_token(0, &user1, email_change_res.id.clone())
        .await;
    assert!(email_change.err_general.get_untracked().is_empty());

    {
        let result = email_change.current_confirm(0_u128.to_be_bytes()).await;
        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    {
        let email_change = EmailChangeState::new(server.client.clone());
        let result = email_change.init(0).await;
        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    let email_change = EmailChangeState::new(server.client.clone());
    email_change.init(email_change_res.id).await.unwrap();
    assert!(email_change.err_general.get_untracked().is_empty());

    email_change
        .current_confirm(current_token.clone())
        .await
        .unwrap();
    assert!(email_change.err_general.get_untracked().is_empty());

    {
        let result = email_change.current_confirm(current_token).await;
        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    {
        let result = email_change.new_add("prime2").await;
        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    let email_change_res = email_change.new_add("prime2@heyadora.com").await.unwrap();

    let new_token = server
        .email_change_get_new_token(0, &user1, email_change_res.id.clone())
        .await;

    {
        let result = email_change.new_confirm(0_u128.to_be_bytes()).await;

        assert!(!email_change.err_general.get_untracked().is_empty());
        assert!(result.is_none());
    }

    email_change.new_confirm(new_token).await.unwrap();
    email_change.finish().await.unwrap();

    let result = server.user_get_by_session_key(session1).await.unwrap();
    assert_eq!(result.email, "prime2@heyadora.com");
}
