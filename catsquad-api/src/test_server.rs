use crate::{
    auth::auth_optional_middleware, proccess_images::storage_file_path, promote_admin, server::app,
    state::AppState,
};
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderName,
    middleware,
};
use catsquad_client::{AxumTestSender, Client};
use catsquad_db::{DbEmailSent, DbEmailSentReason};
use catsquad_shared::{PostState, SessionTokenKind, UUID_ZERO, UserState, Uuid, i64_to_str};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TestServer {
    // app: Router,
    server: Arc<axum_test::TestServer>,
    pub inject_headers: Arc<RwLock<Vec<(HeaderName, String)>>>,
    pub client: Client<AxumTestSender>,
    pub state: AppState,
}

#[derive(Clone, Debug)]
pub struct TestImg {
    pub size: u32,
    pub hash: i64,
    pub saved_path: PathBuf,
    pub storage_path: PathBuf,
    pub thumbnail_path: PathBuf,
}

impl TestServer {
    pub async fn new(time: u64, test_db: &str) -> Self {
        let state = AppState::mem(time, test_db).await;
        let router = app(state.clone())
            .await
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                async move |State(_app_state): State<AppState>,
                            mut req: axum::extract::Request,
                            next: axum::middleware::Next| {
                    let extensions = req.extensions_mut();
                    extensions.insert(ConnectInfo(SocketAddr::V4(SocketAddrV4::new(
                        Ipv4Addr::from_octets([127, 0, 0, 1]),
                        5359,
                    ))));

                    next.run(req).await
                },
            ));
        //
        let server = axum_test::TestServer::new(router);
        let server = Arc::new(server);
        let inject_headers = Arc::new(RwLock::new(Vec::new()));

        let client = Client::new(AxumTestSender::new(server.clone(), inject_headers.clone()));
        Self {
            server,
            state,
            client,
            inject_headers,
        }
    }

    // pub fn origin(&self) -> Option<String> {
    //     self.server.server_address().map(|v| v.to_string())
    // }

    // pub async fn set_session_key(&self, session_key: impl Into<String>) {
    //     *self.client.client.session_key.write().await = session_key.into();
    // }
    pub async fn inject_header(&self, name: HeaderName, value: String) {
        let mut inject_headers = self.inject_headers.write().await;
        inject_headers.push((name, value));
    }

    pub async fn remove_header(&self, name: HeaderName) {
        let mut inject_headers = self.inject_headers.write().await;
        let Some(pos) = inject_headers.iter().position(|v| v.0 == name) else {
            return;
        };
        inject_headers.remove(pos);
    }

    pub async fn email_sent_get_filtered(&self, reason: DbEmailSentReason) -> Vec<DbEmailSent> {
        let reason = reason.to_string();
        self.state
            .db
            .email_sent_get_all()
            .await
            .unwrap()
            .into_iter()
            .filter(|v| v.reason == reason)
            .collect::<Vec<DbEmailSent>>()
    }

    pub async fn create_img_input(&self, i: usize) -> TestImg {
        use crate::get_file_hash_for_testing_by_path;
        use crate::get_file_size;
        use crate::proccess_images::thumbnail_file_path;
        use catsquad_log::prelude::*;
        use catsquad_seed::create_img;

        let tmp_path = self.state.get_tmp_path().await;
        let storage_path = self.state.get_storage_path().await;
        let saved_path = tmp_path.join(format!("input{i}.png"));

        create_img(saved_path.clone(), i.to_string());

        let size = get_file_size(saved_path.as_path()).await;
        let hash = get_file_hash_for_testing_by_path(saved_path.as_path()).await;
        let hash_str = i64_to_str(hash);
        let input_storage_path = storage_file_path(storage_path.as_path(), &hash_str, "png");
        let input_thumbnail_path = thumbnail_file_path(storage_path.as_path(), hash_str);

        trace!("{saved_path:?}");

        assert!(saved_path.exists());
        assert!(!input_storage_path.exists());
        assert!(!input_thumbnail_path.exists());

        TestImg {
            size,
            hash,
            saved_path,
            storage_path: input_storage_path,
            thumbnail_path: input_thumbnail_path,
        }
    }

    pub async fn user_state(&self, username: impl Into<String>, user_state: UserState) -> Uuid {
        let username = username.into();

        self.state
            .db
            .user_update_state(
                async |_| String::new(),
                0,
                SessionTokenKind::Root,
                vec![username.clone()],
                user_state,
            )
            .await
            .unwrap();

        let user = self
            .state
            .db
            .user_get_by_username(SessionTokenKind::Root, &username)
            .await
            .unwrap();

        if user_state == UserState::Active {
            let token = self
                .state
                .db
                .session_add(0, user.email.clone())
                .await
                .unwrap();

            return token.token;
        }

        UUID_ZERO
    }

    pub async fn post_state_active(&self, username: impl Into<String>, post_id: i64) {
        let username = username.into();

        self.state
            .db
            .post_update_state(0, username.clone(), post_id, PostState::Active)
            .await
            .unwrap();
    }

    pub async fn post_state_hidden(&self, username: impl Into<String>, post_id: i64) {
        let username = username.into();

        self.state
            .db
            .post_update_state(0, username.clone(), post_id, PostState::Hidden)
            .await
            .unwrap();
    }

    pub async fn admin(&self, username: impl Into<String>) {
        self.state.set_admin_username(username).await;
        promote_admin(self.state.clone()).await.unwrap();
    }
}
