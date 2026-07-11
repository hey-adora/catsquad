use axum::{
    Router,
    routing::{get, post},
};
use catsquad_log::prelude::*;

use crate::{api, state::AppState};

pub async fn server() {
    let state = AppState::mem().await;
    let addr = state.get_bind().await;
    info!("starting server {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let app = app(state);

    axum::serve(listener, app).await.unwrap();
}

pub fn app(state: AppState) -> Router {
    let router_public = Router::new()
        .route(api::USER_ADD_ENDPOINT, post(api::user_add))
        .route(api::INVITE_ADD_ENDPOINT, post(api::invite_add))
        .route(api::INVITE_GET_BY_KEY_ENDPOINT, get(api::invite_get_by_key))
        .with_state(state);

    router_public
}
