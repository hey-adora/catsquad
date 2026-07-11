use axum::{
    Router,
    body::Body,
    http::{self, Request},
};
use tower::{Service, ServiceExt};

use crate::{server::app, state::AppState};
use catsquad_log::prelude::*;

#[derive(Clone)]
pub struct TestServer {
    app: Router,
    pub state: AppState,
}

impl TestServer {
    pub async fn new() -> Self {
        let state = AppState::mem().await;
        let app = app(state.clone());
        Self { app, state }
    }

    pub async fn get<T: for<'a> serde::Deserialize<'a>>(&self, path: impl AsRef<str>) -> T {
        let path = path.as_ref();
        debug!("CLIENT SEND GET {}", path);

        let req = Request::get(path).body(Body::empty()).unwrap();

        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let body = res.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

        debug!(
            "CLIENT RECV GET {} {} {}",
            status,
            path,
            String::from_utf8_lossy(&bytes)
        );

        serde_json::from_slice(&bytes).unwrap()
    }

    pub async fn post<T: for<'a> serde::Deserialize<'a>>(
        &self,
        path: impl AsRef<str>,
        data: impl Into<String>,
    ) -> T {
        let path = path.as_ref();
        let data = data.into();
        debug!("CLIENT SEND POST {} {}", path, &data);

        let req = Request::post(path)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(Body::from(data))
            .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let body = res.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

        debug!(
            "CLIENT RECV POST {} {} {}",
            status,
            path,
            String::from_utf8_lossy(&bytes)
        );

        serde_json::from_slice(&bytes).unwrap()
    }
}
