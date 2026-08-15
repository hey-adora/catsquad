use catsquad_db::Db;
use rand::distr::SampleString;
use std::{path::PathBuf, sync::Arc};
use tokio::{fs, sync::RwLock};
use url::Url;

use crate::{api_config::ApiConfig, assets::Assets, utils::get_time_ns};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: Db,
    pub assets: Arc<Assets>,
    conf: Arc<RwLock<ApiConfig>>,
    time: Option<Arc<RwLock<u128>>>,
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
            db: Db::mem(0).await,
            conf: Arc::new(RwLock::new(conf)),
            time: Some(Arc::new(RwLock::new(0))),
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
        Self {
            db: Db::local(time, &conf.database_path).await,
            conf: Arc::new(RwLock::new(conf)),
            time: None, // dont set time here, it will never update, used only in tests
            assets: Arc::new(assets),
        }
    }

    pub async fn set_time(&self, new_time: u128) {
        if let Some(time) = &self.time {
            *time.write().await = new_time;
        }
    }

    pub async fn get_time(&self) -> u128 {
        if let Some(time) = &self.time {
            return *time.read().await;
        }

        get_time_ns()
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

    pub async fn get_invite_expiration(&self) -> u128 {
        self.conf.read().await.invite_expiration_ns
    }

    pub async fn get_password_change_expiration(&self) -> u128 {
        self.conf.read().await.password_change_expiration_ns
    }

    pub async fn get_email_change_expiration(&self) -> u128 {
        self.conf.read().await.email_change_expiration_ns
    }

    pub async fn set_email_change_expiration(&self, duration: u128) {
        self.conf.write().await.email_change_expiration_ns = duration;
    }

    pub async fn get_address(&self) -> Url {
        self.conf.read().await.address.clone()
    }

    pub async fn get_bind(&self) -> String {
        self.conf.read().await.bind.clone()
    }
}
