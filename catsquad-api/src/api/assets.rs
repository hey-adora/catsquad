use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};
use catsquad_log::prelude::*;

use crate::state::AppState;

pub async fn index(State(app): State<AppState>) -> impl IntoResponse {
    let bytes = app.assets.index.clone();
    Html(bytes)
}

pub async fn index_404(State(app): State<AppState>) -> impl IntoResponse {
    let bytes = app.assets.index.clone();
    (StatusCode::NOT_FOUND, Html(bytes))
}

pub async fn wasm(State(app): State<AppState>) -> impl IntoResponse {
    let bytes = app.assets.wasm.clone();
    ([(header::CONTENT_TYPE, "application/wasm")], bytes)
}

pub async fn js(State(app): State<AppState>) -> impl IntoResponse {
    let bytes = app.assets.js.clone();
    ([(header::CONTENT_TYPE, "text/javascript")], bytes)
}

pub async fn css(State(app): State<AppState>) -> impl IntoResponse {
    let bytes = app.assets.css.clone();
    ([(header::CONTENT_TYPE, "text/css")], bytes)
}

pub async fn favicon(State(app): State<AppState>) -> impl IntoResponse {
    let bytes = app.assets.favicon.clone();
    ([(header::CONTENT_TYPE, "image/x-icon")], bytes)
}

pub async fn font_hi(State(app): State<AppState>) -> impl IntoResponse {
    let bytes = app.assets.font_hi.clone();
    ([(header::CONTENT_TYPE, "font/woff2")], bytes)
}

pub async fn font_lucky(State(app): State<AppState>) -> impl IntoResponse {
    let bytes = app.assets.font_lucky.clone();
    ([(header::CONTENT_TYPE, "font/ttf")], bytes)
}

// #[tokio::test]
// async fn test_assets() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let (s, status) = server.get_str("/").await;
//     let s = s.unwrap();
//     assert_eq!(status, StatusCode::OK);
//     assert!(s.len() > 0);
// }
