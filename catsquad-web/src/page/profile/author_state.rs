use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::UserGetByUsernameErr;
use leptos::prelude::{RwSignal, Set, Update};
use std::fmt::{Debug, Display};

#[derive(Clone)]
pub struct AuthorState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub stage: RwSignal<AuthorStage>,
    pub username: RwSignal<String>,
    pub support: RwSignal<String>,
    pub aboutme: RwSignal<String>,
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
            aboutme: RwSignal::new(String::new()),
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
        self.support.set(user.support);
        self.aboutme.set(user.aboutme);
        self.created_at.set(user.created_at);
        self.stage.set(AuthorStage::Normal);
    }
}

// <TextEditor
//     id_prefix="decription"
//     title="Decription"
//     text=post_api.description
//     text_length=post_api.live_description_length
//     is_owned=when_is_owner
//     edit_mode_enabled=post_api.update_description_mode
//     max_length=MAX_POST_DESCRIPTION_LENGTH
//     errors=post_api.err_description
//     on_save=edit_description_save
//     node_ref=description_input_editor
//     />

#[cfg(test)]
#[tokio::test]
async fn test_author_state() {
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
}
