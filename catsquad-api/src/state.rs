use catsquad_db::Db;
use rand::distr::SampleString;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api_config::ApiConfig;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: Db,
    conf: Arc<RwLock<ApiConfig>>,
    time: Option<Arc<RwLock<u128>>>,
}

impl AppState {
    pub async fn mem() -> Self {
        let tmp_name = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 16);
        let conf = ApiConfig::new(
            format!("/tmp/catsquad-dev/{tmp_name}/catsquad.conf"),
            format!("/tmp/catsquad-dev/{tmp_name}/db"),
            format!("/tmp/catsquad-dev/{tmp_name}/storage"),
        )
        .await;
        Self {
            db: Db::mem().await,
            conf: Arc::new(RwLock::new(conf)),
            time: Some(Arc::new(RwLock::new(0))),
        }
    }

    pub async fn local() -> Self {
        todo!("need to get correct config and data storage paths")
        // let tmp_name = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 16);
        // let conf = ApiConfig::new(
        //     format!("/tmp/catsquad-dev/{tmp_name}/catsquad.conf"),
        //     format!("/tmp/catsquad-dev/{tmp_name}/db"),
        //     format!("/tmp/catsquad-dev/{tmp_name}/storage"),
        // )
        // .await;
        // Self {
        //     db: Db::mem().await,
        //     conf: Arc::new(RwLock::new(conf)),
        //     time: Some(Arc::new(RwLock::new(0))),
        // }
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

        use std::time::{SystemTime, UNIX_EPOCH};
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        time.as_nanos()
    }

    pub async fn get_invite_expiration(&self) -> u128 {
        self.conf.read().await.invite_expiration_ns
    }

    pub async fn get_address(&self) -> String {
        self.conf.read().await.address.clone()
    }

    pub async fn get_bind(&self) -> String {
        self.conf.read().await.bind.clone()
    }
}
