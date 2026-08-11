use catsquad_log::prelude::*;
use catsquad_shared::{SensitiveUserRes, UserGetBySessionKeyErr};
use catsquad_web_utils::time::time_now_ns;
use leptos::prelude::*;

use crate::page::create_client;

#[derive(Clone, Copy, Default, Debug)]
pub struct PageState {
    pub acc: RwSignal<Option<SensitiveUserRes>>,
    pub acc_pending: RwSignal<bool>,
    pub time: RwSignal<u128>,
}

impl PageState {
    pub fn new() -> Self {
        //
        Self {
            acc_pending: RwSignal::new(true),
            time: RwSignal::new(time_now_ns()),
            ..Default::default()
        }
    }

    pub fn set() {
        provide_context(Self::new());
    }

    pub fn get() -> Self {
        expect_context::<Self>()
    }

    pub fn get_time_ns(&self) -> u128 {
        self.time.get()
    }

    pub fn is_logged_in(&self) -> Option<bool> {
        let pending = self.acc_pending.get();
        let has_data = self.acc.with(|v| v.is_some());
        if pending {
            return None;
        }
        Some(has_data)
    }

    pub fn acc_pending(&self) -> bool {
        self.acc_pending.get()
    }

    pub fn acc_username(&self) -> String {
        self.acc
            .with(|v| v.as_ref().map(|v| v.username.clone()))
            .unwrap_or("error".to_string())
    }

    pub fn user_key(&self) -> String {
        self.acc
            .with(|acc| acc.as_ref().map(|acc| acc.key.clone()))
            .unwrap_or_default()
    }

    pub async fn update_auth(&self) {
        let page = self;
        let client = create_client();
        let result = client
            .user_get_by_session_key()
            .send()
            .await
            .into_json()
            .await;
        match result {
            Ok(user) => {
                page.acc.set(Some(user));
            }
            Err(UserGetBySessionKeyErr::Unauthorized(_)) => {
                page.acc.set(None);
                info!("user is not logged in")
            }
            Err(UserGetBySessionKeyErr::InternalServer) => {
                error!("acc update internal server err")
            }
        }
        page.acc_pending.set(false);
    }
}
