use axum::{
    Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
};
use catsquad_db::DbUser;
use catsquad_log::prelude::*;
use catsquad_shared::{DEFAULT_GLOBAL_MAX_UPLOAD_SIZE, MAX_STORAGE_PER_FILE};
use tokio::fs;

use crate::{
    api::{self, assets::index_404},
    auth::{auth_middleware, auth_optional_middleware},
    state::AppState,
};
pub async fn server() {
    let state = AppState::local().await;
    let addr = state.get_bind().await;
    info!("starting server {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let app = app(state).await;

    axum::serve(listener, app).await.unwrap();
}

pub async fn app(state: AppState) -> Router {
    let router_web = Router::new()
        .route(catsquad_shared::LINK_WEB_INDEX, get(api::assets::index))
        .route(catsquad_shared::LINK_WEB_REGISTER, get(api::assets::index))
        .route(catsquad_shared::LINK_WEB_LOGIN, get(api::assets::index))
        .route(catsquad_shared::LINK_WEB_UPLOAD, get(api::assets::index))
        .route(catsquad_shared::LINK_WEB_SETTINGS, get(api::assets::index))
        .route(catsquad_shared::LINK_WEB_POST, get(api::assets::index));

    let router_assets = Router::new()
        .route(catsquad_shared::LINK_WEB_CSS, get(api::assets::css))
        .route(catsquad_shared::LINK_WEB_WASM, get(api::assets::wasm))
        .route(catsquad_shared::LINK_WEB_JS, get(api::assets::js))
        .route(catsquad_shared::LINK_WEB_FAVICON, get(api::assets::favicon))
        .route(catsquad_shared::LINK_WEB_FONT_HI, get(api::assets::font_hi))
        .route(
            catsquad_shared::LINK_WEB_FONT_LUCKY,
            get(api::assets::font_lucky),
        );

    let mut test_backdoors = Router::new();

    #[cfg(feature = "test_backdoors")]
    {
        test_backdoors = test_backdoors.route(
            catsquad_shared::TEST_BACKDOOR_LINK_API_EMAIL_SENT_GET_ALL,
            get(api::test_backdoor_email_sent_get_all),
        );
    }

    let router_public = Router::new()
        .route(
            catsquad_shared::LINK_API_COMMENT_SEARCH,
            get(api::comment_search),
        )
        .route(catsquad_shared::LINK_API_POST_SEARCH, get(api::post_search))
        .route(
            catsquad_shared::LINK_API_SESSION_ADD,
            post(api::session_add),
        )
        .route(catsquad_shared::LINK_API_USER_ADD, post(api::user_add))
        .route(catsquad_shared::LINK_API_INVITE_ADD, post(api::invite_add))
        .route(
            catsquad_shared::LINK_API_INVITE_GET_BY_KEY,
            get(api::invite_get_by_key),
        );

    let api_router_upload = Router::new()
        .route(
            catsquad_shared::LINK_API_POST_UPDATE_FILE_ADD,
            post(api::post_update_file_add),
        )
        .layer(DefaultBodyLimit::max(DEFAULT_GLOBAL_MAX_UPLOAD_SIZE))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let router_optionanl_auth = Router::new()
        .route(
            catsquad_shared::LINK_API_PASSWORD_CHANGE_ADD,
            post(api::password_change_add),
        )
        .route(
            catsquad_shared::LINK_API_PASSWORD_CHANGE_UPDATE_CONFIRM,
            post(api::user_password_change_confirm),
        )
        .route(
            catsquad_shared::LINK_API_POST_GET_BY_KEY,
            get(api::post_get_by_key),
        )
        .route(
            catsquad_shared::LINK_API_POST_FILE_GET_BY_HASH,
            get(api::post_file_get_by_hash),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_optional_middleware,
        ));

    let router_auth = Router::new()
        .route(
            catsquad_shared::LINK_API_COMMENT_ADD,
            post(api::comment_add),
        )
        .route(
            catsquad_shared::LINK_API_COMMENT_UPDATE_TEXT,
            post(api::comment_update_text),
        )
        .route(
            catsquad_shared::LINK_API_COMMENT_REMOVE,
            post(api::comment_remove),
        )
        .route(catsquad_shared::LINK_API_POST_ADD, post(api::post_add))
        .route(
            catsquad_shared::LINK_API_POST_REMOVE,
            delete(api::post_remove),
        )
        .route(
            catsquad_shared::LINK_API_POST_LIKE_ADD,
            post(api::post_like_add),
        )
        .route(
            catsquad_shared::LINK_API_POST_LIKE_REMOVE,
            post(api::post_like_remove),
        )
        .route(
            catsquad_shared::LINK_API_POST_LIKE_GET_BY_POST,
            get(api::post_like_get_by_post),
        )
        .route(
            catsquad_shared::LINK_API_POST_UPDATE_FILE_REMOVE,
            post(api::post_update_file_remove),
        )
        .route(
            catsquad_shared::LINK_API_POST_UPDATE_TAGS,
            post(api::post_update_tags),
        )
        .route(
            catsquad_shared::LINK_API_POST_UPDATE_STATE,
            post(api::post_update_state),
        )
        .route(
            catsquad_shared::LINK_API_POST_UPDATE_TITLE,
            post(api::post_update_title),
        )
        .route(
            catsquad_shared::LINK_API_POST_UPDATE_DESCRIPTION,
            post(api::post_update_description),
        )
        .route(
            catsquad_shared::LINK_API_EMAIL_CHANGE_ADD,
            post(api::email_change_add),
        )
        .route(
            catsquad_shared::LINK_API_EMAIL_CHANGE_RESEND,
            post(api::email_change_resend),
        )
        .route(
            catsquad_shared::LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_CONFIRM,
            post(api::email_change_update_current_confirm),
        )
        .route(
            catsquad_shared::LINK_API_EMAIL_CHANGE_UPDATE_NEW_ADD,
            post(api::email_change_update_new_add),
        )
        .route(
            catsquad_shared::LINK_API_EMAIL_CHANGE_UPDATE_NEW_CONFIRM,
            post(api::email_change_update_new_confirm),
        )
        .route(
            catsquad_shared::LINK_API_EMAIL_CHANGE_UPDATE_FINISH,
            post(api::email_change_update_finish),
        )
        .route(
            catsquad_shared::LINK_API_EMAIL_CHANGE_UPDATE_CANCEL,
            post(api::email_change_update_cancel),
        )
        .route(
            catsquad_shared::LINK_API_USER_UPDATE_USERNAME,
            post(api::user_update_username),
        )
        .route(
            catsquad_shared::LINK_API_SESSION_REMOVE,
            post(api::session_remove),
        )
        .route(
            catsquad_shared::LINK_API_SESSION_GET_BY_SESSION_KEY,
            get(api::user_get_by_session_token),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .merge(test_backdoors)
        .merge(api_router_upload)
        .merge(router_assets)
        .merge(router_web)
        .merge(router_public)
        .merge(router_auth)
        .merge(router_optionanl_auth)
        .fallback(index_404)
        .with_state(state.clone());

    app
}

// async fn dir_to_mem(path: impl AsRef<str>) -> (Vec<String>, Vec<u8>) {
//     let output_paths = Vec::new();
//     let output_data = Vec::new();
//     let mut paths = fs::read_dir(path.as_ref()).await.unwrap();
//     loop {
//         let Some(path) = paths.next_entry().await.unwrap() else {
//             break;
//         };
//     }

//     (output_paths, output_data)
// }
