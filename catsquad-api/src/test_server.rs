use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use axum::{
    Router,
    body::Body,
    http::{self, Request, Response, StatusCode, header},
};
use axum_test::Transport;
use catsquad_client::{AxumTestSender, Client, Error, Sender};
use catsquad_db::{DbEmailSent, DbEmailSentReason};
use catsquad_shared::ToForm;
use tokio::sync::RwLock;
use tower::{Service, ServiceExt};

use crate::{
    auth::{auth_token_get, create_auth_cookie_str},
    server::app,
    state::AppState,
};
use catsquad_log::prelude::*;

#[derive(Clone)]
pub struct TestServer {
    // app: Router,
    server: Arc<axum_test::TestServer>,
    pub client: Client<AxumTestSender>,
    pub state: AppState,
}

// #[derive(Clone)]
// pub struct TestClient {
//     // app: Router,
//     server: Arc<axum_test::TestServer>,
//     pub session_key: Arc<RwLock<String>>,
// }

// #[derive(Debug)]
// pub struct HttpTestRes<R, E>
// where
//     R: for<'a> serde::Deserialize<'a> + Debug,
//     E: for<'a> serde::Deserialize<'a> + Debug + Default,
// {
//     response: Result<Response<Body>, SendErr>,
//     debug_data: Option<HttpTestResDebug>,
//     phantom: PhantomData<(R, E)>,
// }

// #[derive(Clone, Debug)]
// pub struct HttpTestResDebug {
//     method: catsquad_client::Method,
//     status: http::StatusCode,
//     path: String,
// }

// impl<R, E> HttpTestRes<R, E>
// where
//     R: for<'a> serde::Deserialize<'a> + Debug,
//     E: for<'a> serde::Deserialize<'a> + Debug + Default,
// {
//     pub fn new(res: Result<Response<Body>, SendErr>) -> Self {
//         Self {
//             response: res,
//             debug_data: None,
//             phantom: PhantomData,
//         }
//     }

//     pub fn get_auth_token(&self) -> Option<String> {
//         let headers = self.response.as_ref().ok()?.headers().clone();
//         let token = auth_token_get(&headers, header::SET_COOKIE);
//         token
//     }

//     pub async fn into_res(self) -> Result<R, E> {
//         let body = self.response.map_err(|_err| E::default())?.into_body();
//         let debug_method = self
//             .debug_data
//             .as_ref()
//             .map(|v| v.method.to_string())
//             .unwrap_or_default();
//         let debug_status = self
//             .debug_data
//             .as_ref()
//             .map(|v| v.status.to_string())
//             .unwrap_or_default();
//         let debug_path = self
//             .debug_data
//             .as_ref()
//             .map(|v| v.path.to_string())
//             .unwrap_or_default();

//         let bytes = axum::body::to_bytes(body, usize::MAX)
//             .await
//             .map_err(|_err| E::default())?;
//         let result = serde_json::from_slice(&bytes).map_err(|_err| E::default())?;
//         let debug_lossy = String::from_utf8_lossy(&bytes);
//         debug!(
//             "CLIENT RECV {} {} {}\n{}\n{:#?}",
//             debug_method, debug_status, debug_path, debug_lossy, result
//         );
//         result
//     }
// }

// impl HttpClient for TestClient {
//     type Res<
//         R: for<'a> serde::Deserialize<'a> + Debug,
//         E: for<'a> serde::Deserialize<'a> + Debug + Default,
//     > = HttpTestRes<R, E>;

//     async fn req<R, E, Req>(
//         &self,
//         method: catsquad_client::Method,
//         path: impl AsRef<str>,
//         body_type: BodyType,
//         body: Option<Req>,
//     ) -> Self::Res<R, E>
//     where
//         R: for<'a> serde::Deserialize<'a> + Debug,
//         E: for<'a> serde::Deserialize<'a> + Debug + Default,
//         Req: serde::Serialize + Debug,
//     {
//         let path = path.as_ref();
//         debug!("CLIENT SEND {} {}\n{:#?}", method, path, &body);
//         let inner = async || -> Result<(HttpTestResDebug, Response<Body>), SendErr> {
//             let body = match body {
//                 Some(v) => Body::from(
//                     v.to_form()
//                         .map_err(|err| SendErr::Serialize(err.to_string()))?,
//                 ),
//                 None => Body::empty(),
//             };
//             let req = match method {
//                 catsquad_client::Method::Post => Request::post(path),
//                 catsquad_client::Method::Get => Request::get(path),
//             };
//             let req = match body_type {
//                 BodyType::None => req,
//                 BodyType::Form => req.header(
//                     http::header::CONTENT_TYPE,
//                     "application/x-www-form-urlencoded",
//                 ),
//             };

//             let session_key = self.session_key.read().await;
//             let req = if !session_key.is_empty() {
//                 req.header(header::COOKIE, create_auth_cookie_str(&*session_key))
//             } else {
//                 req
//             };

//             let req = req.body(body).unwrap();
//             let res = self.app.clone().oneshot(req).await.unwrap();
//             let status = res.status();

//             let debug = HttpTestResDebug {
//                 method,
//                 status,
//                 path: path.to_string(),
//             };

//             Ok((debug, res))
//         };

//         let res = inner().await;
//         let res = match res {
//             Ok((debug, res)) => {
//                 let res = Ok(res);
//                 let mut res = Self::Res::new(res);
//                 res.debug_data = Some(debug);
//                 res
//             }
//             Err(err) => {
//                 let res = Err(err);
//                 let res = Self::Res::new(res);
//                 res
//             }
//         };

//         res
//     }
// }

impl TestServer {
    pub async fn new() -> Self {
        let state = AppState::mem().await;
        let router = app(state.clone()).await;
        // let config = axum_test::TestServerConfig {
        //     transport: Some(Transport::HttpRandomPort),
        //     ..Default::default()
        // };
        // let server = axum_test::TestServer::new_with_config(router, config);
        let server = axum_test::TestServer::new(router);
        let server = Arc::new(server);
        // let origin = server.server_address().map(|v| v.to_string()).unwrap();
        // trace!("origin {origin}");
        let client = Client::new(AxumTestSender::new(server.clone()));
        // let client = Client::new(TestClient {
        //     // app: router.clone(),
        //     server: server.clone(),
        //     session_key: Arc::new(RwLock::new(String::new())),
        // });
        Self {
            // app: router,
            server,
            state,
            client,
            // client,
        }
    }

    // pub fn origin(&self) -> Option<String> {
    //     self.server.server_address().map(|v| v.to_string())
    // }

    // pub async fn set_session_key(&self, session_key: impl Into<String>) {
    //     *self.client.client.session_key.write().await = session_key.into();
    // }

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

    // pub async fn get_raw(&self, path: impl AsRef<str>) -> Response<Body> {
    //     let path = path.as_ref();
    //     debug!("CLIENT SEND GET {}", path);

    //     let req = Request::get(path).body(Body::empty()).unwrap();
    //     let res = self.app.clone().oneshot(req).await.unwrap();

    //     debug!("CLIENT RECV GET {} {} {}", res.status(), path, "uwknown");

    //     res
    // }

    // pub async fn get_str(&self, path: impl AsRef<str>) -> (Option<String>, StatusCode) {
    //     let path = path.as_ref();
    //     debug!("CLIENT SEND GET {}", path);

    //     let req = Request::get(path).body(Body::empty()).unwrap();
    //     let res = self.app.clone().oneshot(req).await.unwrap();
    //     let status = res.status();
    //     let body = res.into_body();
    //     let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    //     let s = String::from_utf8(bytes.to_vec()).ok();

    //     debug!("CLIENT RECV GET {} {} {:?}", status, path, s);

    //     (s, status)
    // }

    // pub async fn get<T: for<'a> serde::Deserialize<'a>>(&self, path: impl AsRef<str>) -> T {
    //     let path = path.as_ref();
    //     debug!("CLIENT SEND GET {}", path);

    //     let req = Request::get(path).body(Body::empty()).unwrap();

    //     let res = self.app.clone().oneshot(req).await.unwrap();
    //     let status = res.status();
    //     let body = res.into_body();
    //     let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

    //     debug!(
    //         "CLIENT RECV GET {} {} {}",
    //         status,
    //         path,
    //         String::from_utf8_lossy(&bytes)
    //     );

    //     serde_json::from_slice(&bytes).unwrap()
    // }

    // pub async fn post<T: for<'a> serde::Deserialize<'a>>(
    //     &self,
    //     path: impl AsRef<str>,
    //     data: impl Into<String>,
    // ) -> T {
    //     let path = path.as_ref();
    //     let data = data.into();
    //     debug!("CLIENT SEND POST {} {}", path, &data);

    //     let req = Request::post(path)
    //         .header(
    //             http::header::CONTENT_TYPE,
    //             "application/x-www-form-urlencoded",
    //         )
    //         .body(Body::from(data))
    //         .unwrap();
    //     let res = self.app.clone().oneshot(req).await.unwrap();
    //     let status = res.status();
    //     let body = res.into_body();
    //     let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

    //     debug!(
    //         "CLIENT RECV POST {} {} {}",
    //         status,
    //         path,
    //         String::from_utf8_lossy(&bytes)
    //     );

    //     serde_json::from_slice(&bytes).unwrap()
    // }

    // pub async fn post_auth_empty<T: for<'a> serde::Deserialize<'a>>(
    //     &self,
    //     path: impl AsRef<str>,
    //     session_key: impl AsRef<str>,
    // ) -> (T, StatusCode) {
    //     let session_key = session_key.as_ref();
    //     let path = path.as_ref();
    //     debug!("CLIENT SEND POST AUTH {} Body::empty()", path);

    //     let req = Request::post(path)
    //         .header(header::COOKIE, create_auth_cookie_str(session_key))
    //         .body(Body::empty())
    //         .unwrap();
    //     let res = self.app.clone().oneshot(req).await.unwrap();
    //     let status = res.status();
    //     let body = res.into_body();
    //     let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

    //     debug!(
    //         "CLIENT RECV POST AUTH {} {} {}",
    //         status,
    //         path,
    //         String::from_utf8_lossy(&bytes)
    //     );

    //     (serde_json::from_slice(&bytes).unwrap(), status)
    // }

    // pub async fn post_auth<T: for<'a> serde::Deserialize<'a>>(
    //     &self,
    //     path: impl AsRef<str>,
    //     data: impl Into<String>,
    //     session_key: impl AsRef<str>,
    // ) -> (T, StatusCode) {
    //     let session_key = session_key.as_ref();
    //     let path = path.as_ref();
    //     let data = data.into();
    //     debug!("CLIENT SEND POST AUTH {} {}", path, &data);

    //     let req = Request::post(path)
    //         .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
    //         .header(header::COOKIE, create_auth_cookie_str(session_key))
    //         .body(Body::from(data))
    //         .unwrap();
    //     let res = self.app.clone().oneshot(req).await.unwrap();
    //     let status = res.status();
    //     let body = res.into_body();
    //     let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

    //     debug!(
    //         "CLIENT RECV POST AUTH {} {} {}",
    //         status,
    //         path,
    //         String::from_utf8_lossy(&bytes)
    //     );

    //     (serde_json::from_slice(&bytes).unwrap(), status)
    // }

    // pub async fn get_auth<T: for<'a> serde::Deserialize<'a>>(
    //     &self,
    //     path: impl AsRef<str>,
    //     session_key: impl AsRef<str>,
    //     // data: impl Into<String>,
    // ) -> (T, StatusCode) {
    //     let session_key = session_key.as_ref();
    //     let path = path.as_ref();
    //     // let data = data.into();
    //     debug!("CLIENT SEND GET AUTH {}", path);

    //     let req = Request::get(path)
    //         // .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
    //         .header(header::COOKIE, create_auth_cookie_str(session_key))
    //         .body(Body::empty())
    //         .unwrap();
    //     let res = self.app.clone().oneshot(req).await.unwrap();
    //     let status = res.status();
    //     let body = res.into_body();
    //     let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

    //     debug!(
    //         "CLIENT RECV GET AUTH {} {} {}",
    //         status,
    //         path,
    //         String::from_utf8_lossy(&bytes)
    //     );

    //     (serde_json::from_slice(&bytes).unwrap(), status)
    // }

    // pub async fn post_and_get_auth_token<T: for<'a> serde::Deserialize<'a>>(
    //     &self,
    //     path: impl AsRef<str>,
    //     data: impl Into<String>,
    // ) -> (T, Option<String>) {
    //     let path = path.as_ref();
    //     let data = data.into();
    //     debug!("CLIENT SEND POST {} {}", path, &data);

    //     let req = Request::post(path)
    //         .header(
    //             http::header::CONTENT_TYPE,
    //             "application/x-www-form-urlencoded",
    //         )
    //         .body(Body::from(data))
    //         .unwrap();
    //     let res = self.app.clone().oneshot(req).await.unwrap();
    //     let headers = res.headers().clone();
    //     let token = auth_token_get(&headers, header::SET_COOKIE);
    //     let status = res.status();
    //     let body = res.into_body();
    //     let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

    //     debug!(
    //         "CLIENT RECV POST {} {} {:?} {}",
    //         status,
    //         path,
    //         headers,
    //         String::from_utf8_lossy(&bytes)
    //     );
    //     let result = serde_json::from_slice(&bytes).unwrap();

    //     (result, token)
    // }
}
