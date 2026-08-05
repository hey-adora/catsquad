use catsquad_log::prelude::*;
use catsquad_shared::{
    self as cs, PostFile, PostState, ToForm, link_relative_invite_get_by_key,
    link_relative_post_get_by_key,
};
use http::{HeaderMap, HeaderName, StatusCode, header};
use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

mod sender;

#[cfg(feature = "axum_test")]
pub use sender::axum_test::AxumTestSender;

#[cfg(feature = "reqwest")]
pub use sender::reqwesty::ReqwestSender;

#[cfg(feature = "xml_http_request")]
pub use sender::xml_http_request::XMLSender;

#[derive(Clone)]
pub struct Client<TSender: Sender + Debug + Clone> {
    pub sender: TSender,
}

#[derive(Debug)]
pub struct Builder<TSender, TResult, TError>
where
    TSender: Sender + Debug,
    TSender::TResponse: Response + Debug,
    TResult: for<'a> serde::Deserialize<'a> + Debug,
    TError: for<'a> serde::Deserialize<'a> + Debug + Default,
{
    pub sender: TSender,
    pub params: SenderParams,
    phantom: PhantomData<(TResult, TError)>,
}

pub trait Sender {
    type TResponse;
    fn send(&self, params: &SenderParams) -> impl Future<Output = Result<Self::TResponse, Error>>;
}

#[derive(Debug)]
pub struct ResponseContainer<TResult, TError, TResponse>
where
    TResult: for<'a> serde::Deserialize<'a> + Debug,
    TError: for<'a> serde::Deserialize<'a> + Debug + Default,
{
    pub request: SenderParams,
    pub response: Result<TResponse, Error>,
    phantom: PhantomData<(TResult, TError)>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UploadStats {
    pub total_bytes: u64,
    pub completed_bytes: u64,
    pub completed_precentage: u64,
    pub upload_speed_bytes: u64,
    pub upload_accumulator: u64,
    pub updated_at: u128,
    pub rate_speed_ns: u128,
}

impl UploadStats {
    pub fn new(rate_speed_ns: u128) -> Self {
        // let completed_precentage = ((completed_bytes as f64 / total_bytes as f64) * 100.0) as u64;
        Self {
            total_bytes: 0,
            completed_bytes: 0,
            completed_precentage: 0,
            upload_speed_bytes: 0,
            upload_accumulator: 0,
            updated_at: 0,
            rate_speed_ns,
        }
    }

    pub fn set_total(&mut self, total_bytes: u64) {
        self.total_bytes = total_bytes;
    }

    pub fn update_by_completed_bytes(&mut self, time: u128, completed_bytes: u64) {
        let completed_diff = completed_bytes.saturating_sub(self.completed_bytes);

        // percentage
        {
            self.completed_bytes = completed_bytes;
            let completed_percentage = (completed_bytes as f64 / self.total_bytes as f64) * 100.0;
            let completed_precentage = if completed_percentage.is_finite() {
                completed_percentage as u64
            } else {
                0
            };
            self.completed_precentage = completed_precentage;
        }

        // speed
        {
            let time_diff = time.saturating_sub(self.updated_at);
            trace!("{time_diff}");
            if time_diff >= self.rate_speed_ns {
                trace!("swapping the accumulator");
                self.upload_speed_bytes = self.upload_accumulator;
                self.upload_accumulator = completed_diff;
                self.updated_at = time;
            } else {
                trace!("adding to accumulator");
                self.upload_accumulator += completed_diff;
            }
        }
    }
}

// impl From<web_sys::ProgressEvent> for ProgressEvent {
//     fn from(value: web_sys::ProgressEvent) -> Self {
//         //value.loaded() as u64,
//         Self::new(value.total() as u64)
//     }
// }

#[test]
fn test_progress_event() {
    init_log();

    let mut event = UploadStats::new(1);
    event.set_total(20);
    event.update_by_completed_bytes(0, 10);
    assert_eq!(event.completed_bytes, 10);
    assert_eq!(event.total_bytes, 20);
    assert_eq!(event.completed_precentage, 50);
    assert_eq!(event.upload_speed_bytes, 0);
    assert_eq!(event.upload_accumulator, 10);
    assert_eq!(event.updated_at, 0);
    assert_eq!(event.rate_speed_ns, 1);
    event.update_by_completed_bytes(1, 20);
    assert_eq!(event.completed_bytes, 20);
    assert_eq!(event.total_bytes, 20);
    assert_eq!(event.completed_precentage, 100);
    assert_eq!(event.upload_speed_bytes, 10);
    assert_eq!(event.upload_accumulator, 10);
    assert_eq!(event.updated_at, 1);
    assert_eq!(event.rate_speed_ns, 1);
}

#[derive(Clone)]
pub struct SenderParams {
    pub path: String,
    pub method: Method,
    pub body: Body,
    pub headers: Vec<(HeaderName, String)>,
    // pub on_progress: Option<Arc<RwLock<dyn FnMut()>>>,
    // pub upload_stats: Rc<> ,
    pub on_progress: Option<Rc<RefCell<dyn FnMut(UploadStats)>>>,
}

impl Debug for SenderParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SenderParams")
            .field("path", &self.path)
            .field("method", &self.method)
            .field("body", &self.body)
            .field("headers", &self.headers)
            .finish()
    }
}

impl Default for SenderParams {
    fn default() -> Self {
        Self {
            path: String::default(),
            method: Method::default(),
            // upload_stats: ProgressStats::default(),
            body: Body::None,
            headers: Vec::new(),
            on_progress: None,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    None,
    Form(String),
    MultipartForm(Vec<(String, BodyField)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BodyField {
    File(SchrodingersFile),
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchrodingersFile {
    FilePath(String),
    WebFile(web_sys::File),
}

impl SchrodingersFile {
    pub fn into_file_path(self) -> String {
        match self {
            SchrodingersFile::FilePath(v) => v,
            SchrodingersFile::WebFile(_) => panic!("trying to run web stuff on non-web stuff"),
        }
    }

    pub fn into_web_file(self) -> web_sys::File {
        match self {
            SchrodingersFile::FilePath(_) => panic!("trying to run non-web stuff on web stuff"),
            SchrodingersFile::WebFile(v) => v,
        }
    }
}

impl From<web_sys::File> for SchrodingersFile {
    fn from(value: web_sys::File) -> Self {
        SchrodingersFile::WebFile(value)
    }
}

impl From<String> for SchrodingersFile {
    fn from(value: String) -> Self {
        SchrodingersFile::FilePath(value)
    }
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
        self.response.as_ref().map(|v| v.get_status().clone()).ok()
    }

    pub fn get_headers(&self) -> Option<HeaderMap> {
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

        res
    }
}

impl<TSender, TResult, TError> Builder<TSender, TResult, TError>
where
    TSender: Sender + Debug,
    TSender::TResponse: Response + Debug,
    TResult: for<'a> serde::Deserialize<'a> + Debug,
    TError: for<'a> serde::Deserialize<'a> + Debug + Default,
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

    pub fn on_progress(mut self, f: impl FnMut(UploadStats) + 'static) -> Self {
        self.params.on_progress = Some(Rc::new(RefCell::new(f)));

        // self.params.headers.push((name, value.into()));
        self
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
        Self { sender }
    }

    pub fn post_form<TReq, TResult, TError>(
        &self,
        link: impl Into<String>,
        req: TReq,
    ) -> Builder<TSender, TResult, TError>
    where
        TReq: serde::Serialize,
        TResult: for<'a> serde::Deserialize<'a> + Debug,
        TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    {
        let req = req
            .to_form()
            .inspect_err(|err| error!("serializing failed {err}"))
            .unwrap_or_default();
        let params = SenderParams {
            path: link.into(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn post_form_empty<TResult, TError>(
        &self,
        link: impl Into<String>,
    ) -> Builder<TSender, TResult, TError>
    where
        TResult: for<'a> serde::Deserialize<'a> + Debug,
        TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    {
        let params = SenderParams {
            path: link.into(),
            method: Method::Post,
            body: Body::None,
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn invite_add(
        &self,
        email: impl Into<String>,
    ) -> Builder<TSender, catsquad_shared::InviteRes, catsquad_shared::InviteAddErr> {
        let req = catsquad_shared::InviteAddReq {
            email: email.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: cs::LINK_API_INVITE_ADD.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn invite_get_by_key(
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

    pub fn user_add(
        &self,
        username: impl Into<String>,
        invite_key: impl Into<String>,
        password: impl Into<String>,
    ) -> Builder<TSender, catsquad_shared::SensitiveUserRes, catsquad_shared::UserAddErr> {
        let req = catsquad_shared::UserAddReq {
            username: username.into(),
            password: password.into(),
            invite_key: invite_key.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: cs::LINK_API_USER_ADD.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn user_update_username(
        &self,
        password: impl Into<String>,
        new_username: impl Into<String>,
    ) -> Builder<TSender, cs::UserUpdateUsernameRes, cs::UserUpdateUsernameErr> {
        self.post_form(
            cs::LINK_API_USER_UPDATE_USERNAME,
            cs::UserUpdateUsernameReq {
                password: password.into(),
                new_username: new_username.into(),
            },
        )
    }

    pub fn post_add(
        &self,
        title: impl Into<String>,
        description: impl Into<String>,
        tags: impl Into<String>,
    ) -> Builder<TSender, catsquad_shared::PostRes, catsquad_shared::PostAddErr> {
        let req = catsquad_shared::PostAddReq {
            title: title.into(),
            description: description.into(),
            tags: tags.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: cs::LINK_API_POST_ADD.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn post_update_file_add<F: Into<SchrodingersFile>>(
        &self,
        post_key: impl AsRef<str>,
        files: Vec<F>,
    ) -> Builder<TSender, Vec<PostFile>, catsquad_shared::PostUpdateFileAddErr> {
        let body = files
            .into_iter()
            .enumerate()
            .map(|(i, file)| (format!("file{i}"), BodyField::File(file.into())))
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

    pub fn post_update_file_remove(
        &self,
        post_key: impl Into<String>,
        hash: impl Into<String>,
    ) -> Builder<TSender, PostFile, catsquad_shared::PostUpdateFileRemoveErr> {
        let req = catsquad_shared::PostUpdateFileRemoveReq {
            post_key: post_key.into(),
            hash: hash.into(),
        };
        trace!("input req {req:?}");
        let req = req
            .to_form()
            .inspect_err(|err| error!("serializing failed {err}"))
            .unwrap_or_default();

        let params = SenderParams {
            path: catsquad_shared::link_relative_post_update_file_remove().to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn user_get_by_session_key(
        &self,
    ) -> Builder<TSender, catsquad_shared::SensitiveUserRes, catsquad_shared::UserGetBySessionKeyErr>
    {
        let params = SenderParams {
            path: cs::LINK_API_SESSION_GET_BY_SESSION_KEY.to_string(),
            method: Method::Get,
            body: Body::None,
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn session_add(
        &self,
        email: impl Into<String>,
        password: impl Into<String>,
    ) -> Builder<TSender, catsquad_shared::SensitiveUserRes, catsquad_shared::SessionAddErr> {
        let req = catsquad_shared::SessionAddReq {
            email: email.into(),
            password: password.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: cs::LINK_API_SESSION_ADD.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn session_remove(
        &self,
    ) -> Builder<TSender, catsquad_shared::SessionDeleteRes, catsquad_shared::SessionDeleteErr>
    {
        self.post_form_empty(cs::LINK_API_SESSION_DELETE)
    }

    pub fn post_update_title(
        &self,
        post_key: impl Into<String>,
        new_title: impl Into<String>,
    ) -> Builder<TSender, catsquad_shared::PostRes, catsquad_shared::PostUpdateTitleErr> {
        let req = catsquad_shared::PostUpdateTitleReq {
            post_key: post_key.into(),
            new_title: new_title.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: cs::LINK_API_POST_UPDATE_TITLE.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn post_update_description(
        &self,
        post_key: impl Into<String>,
        new_description: impl Into<String>,
    ) -> Builder<TSender, catsquad_shared::PostRes, catsquad_shared::PostUpdateDescriptionErr> {
        let req = catsquad_shared::PostUpdateDescriptionReq {
            post_key: post_key.into(),
            new_description: new_description.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: cs::LINK_API_POST_UPDATE_DESCRIPTION.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn post_update_tags(
        &self,
        post_key: impl Into<String>,
        new_tags: impl Into<String>,
    ) -> Builder<TSender, catsquad_shared::PostRes, catsquad_shared::PostUpdateTagsErr> {
        let req = catsquad_shared::PostUpdateTagsReq {
            post_key: post_key.into(),
            new_tags: new_tags.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: cs::LINK_API_POST_UPDATE_TAGS.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn post_update_state(
        &self,
        post_key: impl Into<String>,
        new_state: PostState,
    ) -> Builder<TSender, catsquad_shared::PostRes, catsquad_shared::PostUpdateStateErr> {
        let req = catsquad_shared::PostUpdateStateReq {
            post_key: post_key.into(),
            new_state: new_state.into(),
        }
        .to_form()
        .inspect_err(|err| error!("serializing failed {err}"))
        .unwrap_or_default();
        let params = SenderParams {
            path: cs::LINK_API_POST_UPDATE_STATE.to_string(),
            method: Method::Post,
            body: Body::Form(req),
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn post_get_by_key(
        &self,
        post_key: impl AsRef<str>,
    ) -> Builder<TSender, catsquad_shared::PostRes, catsquad_shared::PostGetByKeyErr> {
        let link = link_relative_post_get_by_key(post_key);
        let params = SenderParams {
            path: link,
            method: Method::Get,
            body: Body::None,
            ..Default::default()
        };
        let sender = self.sender.clone();
        Builder::new(sender, params)
    }

    pub fn email_change_add(
        &self,
    ) -> Builder<TSender, catsquad_shared::EmailChangeRes, catsquad_shared::EmailChangeAddErr> {
        self.post_form_empty(cs::LINK_API_EMAIL_CHANGE_ADD)
    }

    pub fn email_change_update_current_confirm(
        &self,
        email_change_key: impl Into<String>,
        token: impl Into<String>,
    ) -> Builder<
        TSender,
        catsquad_shared::EmailChangeRes,
        catsquad_shared::EmailChangeUpdateCurrentConfirmErr,
    > {
        self.post_form(
            cs::LINK_API_EMAIL_CHANGE_UPDATE_CURRENT_CONFIRM,
            catsquad_shared::EmailChangeUpdateCurrentConfirmReq {
                email_change_key: email_change_key.into(),
                token: token.into(),
            },
        )
    }

    pub fn email_change_update_new_add(
        &self,
        email_change_key: impl Into<String>,
        new_email: impl Into<String>,
    ) -> Builder<
        TSender,
        catsquad_shared::EmailChangeRes,
        catsquad_shared::EmailChangeUpdateNewAddErr,
    > {
        self.post_form(
            cs::LINK_API_EMAIL_CHANGE_UPDATE_NEW_ADD,
            catsquad_shared::EmailChangeUpdateNewAddReq {
                email_change_key: email_change_key.into(),
                new_email: new_email.into(),
            },
        )
    }

    pub fn email_change_update_new_confirm(
        &self,
        email_change_key: impl Into<String>,
        token: impl Into<String>,
    ) -> Builder<
        TSender,
        catsquad_shared::EmailChangeRes,
        catsquad_shared::EmailChangeUpdateNewConfirmErr,
    > {
        self.post_form(
            cs::LINK_API_EMAIL_CHANGE_UPDATE_NEW_CONFIRM,
            catsquad_shared::EmailChangeUpdateNewConfirmReq {
                email_change_key: email_change_key.into(),
                token: token.into(),
            },
        )
    }

    pub fn email_change_update_finish(
        &self,
        email_change_key: impl Into<String>,
    ) -> Builder<TSender, cs::EmailChangeRes, cs::EmailChangeUpdateFinishErr> {
        self.post_form(
            cs::LINK_API_EMAIL_CHANGE_UPDATE_FINISH,
            cs::EmailChangeUpdateFinishReq {
                email_change_key: email_change_key.into(),
            },
        )
    }

    pub fn email_change_update_cancel(
        &self,
        email_change_key: impl Into<String>,
    ) -> Builder<TSender, cs::EmailChangeRes, cs::EmailChangeUpdateCancelErr> {
        self.post_form(
            cs::LINK_API_EMAIL_CHANGE_UPDATE_CANCEL,
            cs::EmailChangeUpdateCancelReq {
                email_change_key: email_change_key.into(),
            },
        )
    }

    pub fn password_change_add(
        &self,
        email: impl Into<String>,
    ) -> Builder<TSender, cs::PasswordChangeRes, cs::PasswordChangeAddErr> {
        self.post_form(
            cs::LINK_API_PASSWORD_CHANGE_ADD,
            cs::PasswordChangeAddReq {
                email: email.into(),
            },
        )
    }

    pub fn password_change_update_confirm(
        &self,
        password_change_key: impl Into<String>,
        new_password: impl Into<String>,
    ) -> Builder<TSender, cs::PasswordChangeUpdateConfirmRes, cs::PasswordChangeUpdateConfirmErr>
    {
        self.post_form(
            cs::LINK_API_PASSWORD_CHANGE_UPDATE_CONFIRM,
            cs::PasswordChangeUpdateConfirmReq {
                password_change_key: password_change_key.into(),
                new_password: new_password.into(),
            },
        )
    }

    pub fn comment_add(
        &self,
        post_key: impl Into<String>,
        comment_parent_key: Option<impl Into<String>>,
        text: impl Into<String>,
    ) -> Builder<TSender, cs::CommentRes, cs::CommentAddErr> {
        self.post_form(
            cs::LINK_API_COMMENT_ADD,
            cs::CommentAddReq {
                post_key: post_key.into(),
                comment_key: comment_parent_key.map(|v| v.into()),
                text: text.into(),
            },
        )
    }

    pub fn comment_update_text(
        &self,
        comment_key: impl Into<String>,
        text: impl Into<String>,
    ) -> Builder<TSender, cs::CommentRes, cs::CommentUpdateTextErr> {
        self.post_form(
            cs::LINK_API_COMMENT_UPDATE_TEXT,
            cs::CommentUpdateTextReq {
                comment_key: comment_key.into(),
                text: text.into(),
            },
        )
    }

    pub fn comment_remove(
        &self,
        comment_key: impl Into<String>,
    ) -> Builder<TSender, (), cs::CommentRemoveErr> {
        self.post_form(
            cs::LINK_API_COMMENT_REMOVE,
            cs::CommentRemoveReq {
                comment_key: comment_key.into(),
            },
        )
    }
}
