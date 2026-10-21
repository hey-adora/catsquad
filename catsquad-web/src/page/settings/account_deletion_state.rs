use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::Uuid;
use leptos::prelude::*;
use std::fmt::Debug;

#[derive(Clone)]
pub struct AccountDeletionState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub err_general: RwSignal<String>,
    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Copy for AccountDeletionState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

impl<TSender> AccountDeletionState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            client: StoredValue::new_local(client),
        }
    }

    pub async fn send_confirmation(&self) -> Option<()> {
        let Some(client) = self.client.try_get_value() else {
            return None;
        };

        self.err_general.try_update(|v| v.clear());

        let result = client
            .token_user_remove_add()
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => Some(()),
            Err(err) => {
                error!("account_deletion_state fn_send_confirmation {err}");
                self.err_general.try_set(err.to_string());
                None
            }
        }
    }

    pub async fn confirm(&self, token: Uuid) -> Option<()> {
        let Some(client) = self.client.try_get_value() else {
            return None;
        };

        self.err_general.try_update(|v| v.clear());

        let result = client
            .user_remove_by_token(token)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(_) => Some(()),
            Err(err) => {
                error!("account_deletion_state fn_confirm {err}");
                self.err_general.try_set(err.to_string());
                None
            }
        }
    }
}
#[cfg(test)]
#[tokio::test]
async fn test_account_deletion_state() {
    use catsquad_api::{auth::create_auth_cookie_str, utils::rng_str};
    use catsquad_shared::{MAX_USERNAME_LENGTH, PostState, uuid_to_str};
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_account_deletion_state").await;

    let (_user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    let account_deletion = AccountDeletionState::new(server.client.clone());

    // assert error
    {
        assert!(account_deletion.err_general.get_untracked().is_empty());

        let result = account_deletion.send_confirmation().await;

        assert!(result.is_none());
        assert!(!account_deletion.err_general.get_untracked().is_empty());
    }

    server
        .inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session1)),
        )
        .await;

    // assert success send
    let token = {
        let result = account_deletion.send_confirmation().await;

        assert!(result.is_some());
        assert!(account_deletion.err_general.get_untracked().is_empty());

        let tokens = server.state.db.token_get_all().await.unwrap();
        assert_eq!(tokens.len(), 1);

        tokens[0].clone()
    };

    // assert success confirm
    {
        let result = account_deletion.confirm(token.id).await;

        let users = server.state.db.user_get_all().await.unwrap();

        assert!(result.is_some());
        assert_eq!(users.len(), 0);
    }
}
