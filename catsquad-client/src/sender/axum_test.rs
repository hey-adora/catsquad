use std::{fmt::Debug, os::linux::fs, path::Path, sync::Arc};

use crate::{Body, BodyField, Error, Method, Response, ResponseContainer, Sender, SenderParams};
use axum_test::{
    TestServer,
    multipart::{MultipartForm, Part},
};
use catsquad_log::prelude::*;
use http::HeaderName;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct AxumTestSender {
    pub server: Arc<TestServer>,
    pub inject_headers: Arc<RwLock<Vec<(HeaderName, String)>>>,
}

impl AxumTestSender {
    pub fn new(
        test_server: Arc<TestServer>,
        inject_headers: Arc<RwLock<Vec<(HeaderName, String)>>>,
    ) -> Self {
        Self {
            server: test_server,
            inject_headers,
        }
    }
}

#[derive(Debug)]
pub struct AxumTestWrap(axum_test::TestResponse);

impl Response for AxumTestWrap {
    fn get_status(&self) -> http::StatusCode {
        self.0.status_code()
    }
    fn get_headers(&self) -> http::HeaderMap {
        self.0.headers().clone()
    }
    async fn into_bytes(self) -> Result<Vec<u8>, Error> {
        Ok(self.0.into_bytes().to_vec())
    }
}

impl Sender for AxumTestSender {
    type TResponse = AxumTestWrap;
    async fn send(&self, params: &SenderParams) -> Result<Self::TResponse, Error> {
        let path = &params.path;
        let method = &params.method;
        let body = &params.body;

        let headers0 = self.inject_headers.read().await.clone();
        let headers1 = params.headers.clone();
        let headers = [headers0, headers1].concat();

        debug!("CLIENT SEND POST\n{}\n{:#?}", path, body);
        let inner = async || -> Result<axum_test::TestResponse, Error> {
            let builder = match method {
                Method::Post => self.server.post(path),
                Method::Get => self.server.get(path),
            };

            let builder = match body {
                Body::None => builder,
                Body::Form(body) => {
                    let builder = builder.add_header(
                        http::header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    );
                    let body = body.as_bytes().to_vec();
                    builder.bytes(body.into())
                }
                // .body(body.clone()),
                Body::MultipartForm(multi_form) => {
                    let multi_form = multi_form.clone();
                    let mut form = MultipartForm::new();
                    // let mut form = reqwest::multipart::Form::new();
                    for (name, value) in multi_form {
                        match value {
                            BodyField::File(file) => {
                                let file = Path::new(&file);
                                let file_name =
                                    file.file_name().unwrap().to_string_lossy().to_string();
                                let file = tokio::fs::read(file).await.unwrap();
                                let file = Part::bytes(file).file_name(file_name);
                                form = form.add_part(name, file);
                                // .mime_type(&"text/markdown");
                                // .await
                                // .map_err(|err| Error::Serialize(err.to_string()))?;
                            }
                            BodyField::Text(text) => {
                                form = form.add_text(name, text);
                            }
                            BodyField::Bytes(bytes) => {
                                let file = Part::bytes(bytes);
                                form = form.add_part(name, file);
                                // form = form.add_part(name, reqwest::multipart::Part::bytes(bytes));
                            }
                        }
                    }
                    builder.multipart(form)
                }
            };

            let builder = headers.into_iter().fold(builder, |builder, (name, val)| {
                builder.add_header(name, val)
            });

            let res = builder.await;
            // .send()
            // .inspect_err(|err| error!("client post err {err}"))
            // .map_err(|err| Error::Send(err.to_string()))?;

            Ok(res)
        };

        let res = inner().await.map(|v| AxumTestWrap(v));

        res
    }
}
