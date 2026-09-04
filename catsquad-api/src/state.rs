use catsquad_db::Db;
use rand::distr::SampleString;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock as SyncRwLock},
};
use tokio::fs;
use tokio::sync::RwLock as AsyncRwLock;
use url::Url;

use crate::{
    api_config::ApiConfig,
    assets::Assets,
    utils::{get_time_micro, get_time_ns},
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: Db,
    pub assets: Arc<Assets>,
    conf: Arc<AsyncRwLock<ApiConfig>>,
    time: Option<Arc<SyncRwLock<u128>>>,
}

impl AppState {
    pub async fn mem() -> Self {
        let tmp_name = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 16);
        fs::create_dir_all(format!("/tmp/catsquad-dev/{tmp_name}/assets"))
            .await
            .unwrap();

        let conf = ApiConfig::new_with_path_override(
            format!("/tmp/catsquad-dev/{tmp_name}/catsquad.conf"),
            format!("/tmp/catsquad-dev/{tmp_name}/db"),
            format!("/tmp/catsquad-dev/{tmp_name}/storage"),
            format!("/tmp/catsquad-dev/{tmp_name}/assets"),
            format!("/tmp/catsquad-dev/{tmp_name}/tmp"),
            // assets_path,
        )
        .await;

        let assets = Assets::mem();

        Self {
            db: Db::test_db(0, "catsquad").await,
            conf: Arc::new(AsyncRwLock::new(conf)),
            time: Some(Arc::new(SyncRwLock::new(0))),
            assets: Arc::new(assets),
        }
    }

    pub async fn local() -> Self {
        let conf = ApiConfig::new("catsquad.conf").await;
        let assets_path = std::env::var("CATSQUAD_WEB_LIB")
            .map(|v| PathBuf::from(v))
            .unwrap_or(conf.assets_path.clone());
        let assets = Assets::new(&assets_path).await;
        let time = get_time_ns();
        panic!("ADD NORMAL AUTH DB LOGIN");
        Self {
            db: Db::test_db(0, "catsquad").await,
            conf: Arc::new(AsyncRwLock::new(conf)),
            time: None, // dont set time here, it will never update, used only in tests
            assets: Arc::new(assets),
        }
    }

    pub fn set_time(&self, new_time: u128) {
        if let Some(time) = &self.time {
            *time.write().unwrap() = new_time;
        }
    }

    pub fn get_time_ns(&self) -> u128 {
        if let Some(time) = &self.time {
            return *time.read().unwrap();
        }

        get_time_ns()
    }

    pub fn get_time_micro(&self) -> u64 {
        if let Some(time) = &self.time {
            return *time.read().unwrap() as u64;
        }

        get_time_micro() as u64
    }

    pub async fn get_secret(&self) -> String {
        // TODO is this even used?
        self.conf.read().await.secret.clone()
    }

    pub async fn get_tmp_path(&self) -> PathBuf {
        self.conf.read().await.tmp_path.clone()
    }

    pub async fn get_storage_path(&self) -> PathBuf {
        self.conf.read().await.storage_path.clone()
    }

    pub async fn get_assets_path(&self) -> PathBuf {
        self.conf.read().await.assets_path.clone()
    }

    // pub async fn get_invite_expiration_ns(&self) -> u128 {
    //     self.conf.read().await.invite_expiration_micros
    // }

    pub async fn get_invite_expiration_micro(&self) -> u64 {
        (self.conf.read().await.invite_expiration_micros / 1000) as u64
    }

    // pub async fn get_password_change_expiration_ns(&self) -> u128 {
    //     self.conf.read().await.password_change_expiration_micros
    // }

    pub async fn get_password_change_expiration_micro(&self) -> u64 {
        (self.conf.read().await.password_change_expiration_micros / 1000) as u64
    }

    pub async fn get_email_change_expiration_micro(&self) -> u64 {
        self.conf.read().await.email_change_expiration_micros
    }

    // pub async fn get_email_change_expiration_(&self) -> u128 {
    //     self.conf.read().await.email_change_expiration_ns
    // }

    pub async fn set_email_change_expiration(&self, duration: u64) {
        self.conf.write().await.email_change_expiration_micros = duration;
    }

    pub async fn get_address(&self) -> Url {
        self.conf.read().await.address.clone()
    }

    pub async fn get_bind(&self) -> String {
        self.conf.read().await.bind.clone()
    }
}
