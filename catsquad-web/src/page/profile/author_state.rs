use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::UserGetByUsernameErr;
use leptos::prelude::{RwSignal, Set, Update};
use std::fmt::{Debug, Display};

use crate::page::create_client;

#[derive(Clone)]
pub struct AuthorState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub stage: RwSignal<AuthorStage>,
    pub username: RwSignal<String>,

    pub support: RwSignal<String>,
    pub err_support: RwSignal<String>,
    pub support_edit_mode: RwSignal<bool>,
    pub live_support_length: RwSignal<usize>,

    pub aboutme: RwSignal<String>,
    pub err_aboutme: RwSignal<String>,
    pub aboutme_edit_mode: RwSignal<bool>,
    pub live_aboutme_length: RwSignal<usize>,

    pub created_at: RwSignal<u64>,
    pub err_general: RwSignal<String>,
    client: Client<TSender>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum AuthorStage {
    Loading,
    Normal,
    NotFound,
    Error,
}

impl<TSender> AuthorState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            stage: RwSignal::new(AuthorStage::Loading),
            username: RwSignal::new(String::new()),

            support: RwSignal::new(String::new()),
            err_support: RwSignal::new(String::new()),
            support_edit_mode: RwSignal::new(false),
            live_support_length: RwSignal::new(0),

            aboutme: RwSignal::new(String::new()),
            err_aboutme: RwSignal::new(String::new()),
            aboutme_edit_mode: RwSignal::new(false),
            live_aboutme_length: RwSignal::new(0),

            err_general: RwSignal::new(String::new()),
            created_at: RwSignal::new(0),
            client,
        }
    }

    pub async fn init(&self, username: impl Into<String>) {
        let username = username.into();

        let result = self
            .client
            .user_get_by_username(&username)
            .send()
            .await
            .into_json()
            .await;

        let user = match result {
            Ok(user) => user,
            Err(UserGetByUsernameErr::NotFound) => {
                self.username.set(username);
                self.support.update(|v| v.clear());
                self.aboutme.update(|v| v.clear());
                self.err_general.set("not found".to_string());
                self.stage.set(AuthorStage::NotFound);
                return;
            }
            Err(err) => {
                self.stage.set(AuthorStage::Error);
                self.err_general.set(err.to_string());
                error!("author_state_init {err}");
                return;
            }
        };

        self.username.set(user.username);
        self.live_support_length.set(user.support.len());
        self.support.set(user.support);
        self.live_aboutme_length.set(user.aboutme.len());
        self.aboutme.set(user.aboutme);
        self.created_at.set(user.created_at);
        self.stage.set(AuthorStage::Normal);
    }

    pub async fn update_support(&self, new_support: impl Into<String>) {
        let new_support = new_support.into();
        let new_support_len = new_support.len();
        self.err_support.update(|v| v.clear());

        let result = self
            .client
            .user_update_support(new_support.clone())
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => (),
            Err(err) => {
                error!("update_support error {err:#?}");
                self.err_support.set(err.to_string());
                return;
            }
        }

        self.support.set(new_support);
        self.live_support_length.set(new_support_len);
        self.support_edit_mode.set(false);
    }

    pub async fn update_aboutme(&self, new_aboutme: impl Into<String>) {
        let new_aboutme = new_aboutme.into();
        let new_aboutme_len = new_aboutme.len();
        self.err_aboutme.update(|v| v.clear());

        let result = self
            .client
            .user_update_aboutme(new_aboutme.clone())
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => (),
            Err(err) => {
                error!("update_support error {err:#?}");
                self.err_aboutme.set(err.to_string());
                return;
            }
        }

        self.aboutme.set(new_aboutme);
        self.live_aboutme_length.set(new_aboutme_len);
        self.aboutme_edit_mode.set(false);
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_author_state() {
    use catsquad_api::auth::create_auth_cookie_str;
    use catsquad_shared::uuid_to_str;
    use http::header;
    use leptos::prelude::GetUntracked;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_post_like_state").await;

    let (_user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    let author_state = AuthorState::new(server.client.clone());

    assert_eq!(author_state.username.get_untracked(), "");
    assert_eq!(author_state.created_at.get_untracked(), 0);
    assert_eq!(author_state.err_general.get_untracked(), "");
    assert_eq!(author_state.stage.get_untracked(), AuthorStage::Loading);

    author_state.init("prime1").await;

    assert_eq!(author_state.username.get_untracked(), "prime1");
    assert_eq!(author_state.created_at.get_untracked(), 0);
    assert_eq!(author_state.err_general.get_untracked(), "");
    assert_eq!(author_state.stage.get_untracked(), AuthorStage::Normal);

    author_state.init("prime2").await;

    assert_eq!(author_state.username.get_untracked(), "prime2");
    assert_eq!(author_state.created_at.get_untracked(), 0);
    assert_eq!(author_state.err_general.get_untracked(), "not found");
    assert_eq!(author_state.stage.get_untracked(), AuthorStage::NotFound);

    author_state.support_edit_mode.set(true);
    author_state.aboutme_edit_mode.set(true);

    author_state.update_support("support1").await;
    author_state.update_aboutme("aboutme1").await;

    assert_eq!(author_state.support.get_untracked(), "");
    assert!(!author_state.err_support.get_untracked().is_empty());
    assert!(author_state.support_edit_mode.get_untracked());
    assert_eq!(author_state.live_support_length.get_untracked(), 0);

    assert_eq!(author_state.aboutme.get_untracked(), "");
    assert!(!author_state.err_aboutme.get_untracked().is_empty());
    assert!(author_state.aboutme_edit_mode.get_untracked());
    assert_eq!(author_state.live_aboutme_length.get_untracked(), 0);

    // TODO figure out wtf is !0

    server
        .inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session1)),
        )
        .await;

    author_state.update_support("support1").await;

    assert_eq!(author_state.support.get_untracked(), "support1");
    assert!(author_state.err_support.get_untracked().is_empty());
    assert!(!author_state.support_edit_mode.get_untracked());
    assert_eq!(
        author_state.live_support_length.get_untracked(),
        "support1".len()
    );

    author_state.update_aboutme("aboutme1").await;
    assert_eq!(author_state.aboutme.get_untracked(), "aboutme1");
    assert!(author_state.err_aboutme.get_untracked().is_empty());
    assert!(!author_state.aboutme_edit_mode.get_untracked());
    assert_eq!(
        author_state.live_aboutme_length.get_untracked(),
        "aboutme1".len()
    )
}
