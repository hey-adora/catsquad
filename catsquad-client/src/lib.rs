use catsquad_log::prelude::*;
use catsquad_shared::{ToForm, link_relative_invite_get_by_key};
use http::{HeaderMap, HeaderName, StatusCode, header};
use std::{fmt::Debug, marker::PhantomData};

mod sender;

#[cfg(feature = "axum_test")]
pub use sender::axum_test::AxumTestSender;

#[cfg(feature = "reqwest")]
pub use sender::reqwesty::ReqwestSender;

#[cfg(feature = "xml_http_request")]
pub use sender::xml_http_request::XMLSender;

#[derive(Clone)]
pub struct Client<TSender: Sender + Debug + Clone> {
    // origin: url::Url,
    pub sender: TSender,
}

#[derive(Debug)]
pub struct Builder<TSender, TResult, TError>
where
    TSender: Sender + Debug,
    TSender::TResponse: Response + Debug,
    TResult: for<'a> serde::Deserialize<'a> + Debug,
    TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    // TRequest: serde::Serialize + Debug,
{
    pub sender: TSender,
    pub params: SenderParams,
    phantom: PhantomData<(TResult, TError)>,
}

pub trait Sender {
    type TResponse;
    // where
    //     Self::TResponse: Response + Debug;
    // type ResponseContainer<TResult, TError, TRequest>
    // where
    //     TResult: for<'a> serde::Deserialize<'a> + Debug,
    //     TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    //     TRequest: serde::Serialize + Debug
    // ;

    fn send(&self, params: &SenderParams) -> impl Future<Output = Result<Self::TResponse, Error>>;
    // fn send<TResult, TError>(
    //     &self,
    //     params: SenderParams,
    // ) -> impl Future<Output = ResponseContainer<TResult, TError, Self::TResponse>>
    // where
    //     TResult: for<'a> serde::Deserialize<'a> + Debug,
    //     TError: for<'a> serde::Deserialize<'a> + Debug + Default;

    // TResponse: Response + Debug;
    // TRequest: serde::Serialize + Debug,
}

#[derive(Debug)]
pub struct ResponseContainer<TResult, TError, TResponse>
where
    TResult: for<'a> serde::Deserialize<'a> + Debug,
    TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    // TRequest: serde::Serialize + Debug,
    // TResponse: Response + Debug,
{
    pub request: SenderParams,
    pub response: Result<TResponse, Error>,
    phantom: PhantomData<(TResult, TError)>,
}

// pub struct SenderParams<TRequest>
// where
//     TRequest: serde::Serialize + Debug,
#[derive(Debug)]
pub struct SenderParams {
    // pub origin: url::Url,
    pub path: String,
    pub method: Method,
    pub body: Body,
    pub headers: Vec<(HeaderName, String)>,
}

// impl<TRequest> Default for SenderParams<TRequest>
// where
//     TRequest: serde::Serialize + Debug,
impl Default for SenderParams {
    fn default() -> Self {
        Self {
            // origin: url::Url::parse("http://localhost:3000").unwrap(),
            path: String::default(),
            method: Method::default(),
            body: Body::None,
            headers: Vec::new(),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("serialization failed {0}")]
    Serialize(String),

    #[error("deserialization failed {0}")]
    Deserialization(String),

    #[error("req send failed {0}")]
    Send(String),
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Method {
    #[default]
    Get,
    Post,
}

#[derive(
    Debug,
    // Default,
    // Clone,
    PartialEq,
    PartialOrd,
    // serde::Serialize,
    // serde::Deserialize,
    // strum::EnumString,
    // strum::Display,
    // strum::EnumIter,
    // strum::EnumIs,
)]
// pub enum Body<TRequest>
// where
//     TRequest: serde::Serialize + Debug,
pub enum Body {
    None,
    Form(String),
    MultipartForm(Vec<(String, BodyField)>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum BodyField {
    File(String),
    Text(String),
    Bytes(Vec<u8>),
}

pub trait Response {
    fn get_status(&self) -> StatusCode;
    fn get_headers(&self) -> HeaderMap;
    fn into_bytes(self) -> impl Future<Output = Result<Vec<u8>, Error>>;
}

impl<TResult, TError, TResponse> ResponseContainer<TResult, TError, TResponse>
where
    TResult: for<'a> serde::Deserialize<'a> + Debug,
    TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    // TRequest: serde::Serialize + Debug,
    TResponse: Response + Debug,
{
    pub fn new(req: SenderParams, res: Result<TResponse, Error>) -> Self {
        Self {
            request: req,
            response: res,
            phantom: PhantomData,
        }
    }

    pub fn get_status(&self) -> Option<StatusCode> {
        // self.response.get_headers()
        self.response.as_ref().map(|v| v.get_status().clone()).ok()
    }

    pub fn get_headers(&self) -> Option<HeaderMap> {
        // self.response.get_headers()
        self.response.as_ref().map(|v| v.get_headers().clone()).ok()
    }

    pub async fn into_res(self) -> Result<TResult, TError> {
        let path = self.request.path;
        let res = self.response.map_err(|_err| TError::default())?;
        let status_code = res.get_status();

        let bytes = res
            .into_bytes()
            .await
            .inspect_err(|err| error!("client err getting bytes {err}"))
            .map_err(|_err| TError::default())?;

        let raw_str = String::from_utf8_lossy(&bytes);
        let res = serde_json::from_slice::<Result<TResult, TError>>(&bytes)
            .inspect_err(|err| error!("client err parsing to json {err}"))
            .map_err(|_err| TError::default());

        debug!(
            "CLIENT RECV {}\n{}\n{}\n{:#?}",
            status_code, path, raw_str, res
        );

        let res = res?;
        // let res = res
        //     .json::<Result<R, E>>()
        //     .await
        //     .inspect_err(|err| error!("client post err {err}"))
        //     .map_err(|_err| E::default())?;

        res
    }
}

impl<TSender, TResult, TError> Builder<TSender, TResult, TError>
where
    TSender: Sender + Debug,
    TSender::TResponse: Response + Debug,
    TResult: for<'a> serde::Deserialize<'a> + Debug,
    TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    // TRequest: serde::Serialize + Debug,
    // TResponse: Response + Debug,
{
    pub fn new(sender: TSender, params: SenderParams) -> Self {
        Self {
            sender,
            params,
            phantom: PhantomData,
        }
    }

    pub async fn send(self) -> ResponseContainer<TResult, TError, TSender::TResponse> {
        debug!("CLIENT SEND \n{}\n{:#?}", self.params.path, self.params);
        let res = self.sender.send(&self.params).await;
        ResponseContainer::new(self.params, res)
    }

    pub fn header_add(mut self, name: header::HeaderName, value: impl Into<String>) -> Self {
        self.params.headers.push((name, value.into()));
        self
    }

    pub fn header_remove(mut self, name: header::HeaderName) -> Self {
        let pos = self.params.headers.iter().position(|v| v.0 == name);
        if let Some(pos) = pos {
            self.params.headers.remove(pos);
        }
        self
    }
}

impl<TSender> Client<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub fn new(sender: TSender) -> Self {
        Self {
            // origin: url::Url::parse(origin.as_ref()).unwrap(),
            sender,
        }
    }

    pub async fn invite_add(
        &self,
        email: impl Into<String>,
    ) -> Builder<
        TSender,
        catsquad_shared::InviteRes,
        catsquad_shared::InviteAddErr,
        // TSender::ResponseContainer,
    > {
        let req = catsquad_shared::InviteAddReq {
            email: email.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: catsquad_shared::LINK_API_INVITE_ADD.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub async fn invite_get_by_key(
        &self,
        invite_key: impl AsRef<str>,
    ) -> Builder<TSender, catsquad_shared::InviteGetByKeyRes, catsquad_shared::InviteGetByKeyErr>
    {
        let link = link_relative_invite_get_by_key(invite_key);
        let params = SenderParams {
            path: link,
            method: Method::Get,
            body: Body::None,
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub async fn user_add(
        &self,
        username: impl Into<String>,
        invite_key: impl Into<String>,
        password: impl Into<String>,
    ) -> Builder<
        TSender,
        catsquad_shared::SensitiveUserRes,
        catsquad_shared::UserAddErr,
        // catsquad_shared::UserAddReq,
    > {
        let req = catsquad_shared::UserAddReq {
            username: username.into(),
            password: password.into(),
            invite_key: invite_key.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: catsquad_shared::LINK_API_USER_ADD.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub async fn post_add(
        &self,
        title: impl Into<String>,
        description: impl Into<String>,
        tags: impl Into<String>,
    ) -> Builder<
        TSender,
        catsquad_shared::PostRes,
        catsquad_shared::PostAddErr,
        // catsquad_shared::PostAddReq,
        // TSender::ResponseContainer,
    > {
        let req = catsquad_shared::PostAddReq {
            title: title.into(),
            description: description.into(),
            tags: tags.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: catsquad_shared::LINK_API_POST_ADD.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub async fn post_update_file_add(
        &self,
        post_key: impl AsRef<str>,
        files: Vec<String>,
    ) -> Builder<
        TSender,
        catsquad_shared::PostRes,
        catsquad_shared::PostUpdateFileAddErr,
        // catsquad_shared::PostUpdateFileAddReq,
        // TSender::ResponseContainer,
    > {
        let body = files
            .into_iter()
            .enumerate()
            .map(|(i, file)| (format!("file{i}"), BodyField::File(file)))
            .collect::<Vec<(String, BodyField)>>();

        let params = SenderParams {
            path: catsquad_shared::link_relative_post_update_file_add(post_key),
            method: Method::Post,
            body: Body::MultipartForm(body),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub async fn user_by_session_key(
        &self,
    ) -> Builder<
        TSender,
        catsquad_shared::SensitiveUserRes,
        catsquad_shared::UserGetBySessionKeyErr,
        // (),
        // TSender::ResponseContainer,
    > {
        let params = SenderParams {
            path: catsquad_shared::LINK_API_SESSION_GET_BY_SESSION_KEY.to_string(),
            method: Method::Get,
            body: Body::None,
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub async fn session_add(
        &self,
        email: impl Into<String>,
        password: impl Into<String>,
    ) -> Builder<
        TSender,
        catsquad_shared::SensitiveUserRes,
        catsquad_shared::SessionAddErr,
        // catsquad_shared::SessionAddReq,
        // TSender::ResponseContainer,
    > {
        let req = catsquad_shared::SessionAddReq {
            email: email.into(),
            password: password.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: catsquad_shared::LINK_API_SESSION_ADD.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }
}
