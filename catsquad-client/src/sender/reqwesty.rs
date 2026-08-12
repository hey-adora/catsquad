use std::fmt::Debug;

use crate::{Body, BodyField, Error, Method, Response, ResponseContainer, Sender, SenderParams};
use catsquad_log::prelude::*;

#[derive(Clone, Debug)]
pub struct ReqwestSender {
    pub origin: url::Url,
}

impl ReqwestSender {
    pub fn new(origin: impl AsRef<str>) -> Self {
        let url = url::Url::parse(origin.as_ref()).unwrap();
        Self { origin: url }
    }
}

#[derive(Debug)]
pub struct ReqwestWrap(reqwest::Response);

impl Response for ReqwestWrap {
    fn get_status(&self) -> http::StatusCode {
        self.0.status()
    }
    fn get_headers(&self) -> http::HeaderMap {
        self.0.headers().clone()
    }
    async fn into_bytes(self) -> Result<Vec<u8>, Error> {
        self.0
            .bytes()
            .await
            .map_err(|err| Error::Deserialization(err.to_string()))
            .map(|v| v.to_vec())
    }
}

impl Sender for ReqwestSender {
    type TResponse = ReqwestWrap;
    // type ResponseContainer<
    //     R: for<'a> serde::Deserialize<'a> + Debug,
    //     E: for<'a> serde::Deserialize<'a> + Debug + Default,
    //     Req: serde::Serialize + Debug,
    //     // ReqwestWrap,
    //     // Wrap: ResponseWrap + Debug,
    // > = ResponseContainer<R, E, Req, ReqwestWrap>;

    // async fn send<TResult, TError>(
    //     &self,
    //     params: SenderParams,
    // ) -> ResponseContainer<TResult, TError, Self::TResponse>
    // where
    //     TResult: for<'a> serde::Deserialize<'a> + Debug,
    //     TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    async fn send(&self, params: &SenderParams) -> Result<Self::TResponse, Error> {
        // let origin = self.origin.clone();
        // let params = params.clone();
        let origin = &self.origin;
        let path = &params.path;
        let method = &params.method;
        let body = &params.body;
        // let body_type = &params.body;
        let headers = &params.headers;

        let path = origin.join(&path).unwrap();
        // let path = gen_url(path);
        // debug!("CLIENT SEND POST\n{}\n{:#?}", path, body);
        let inner = async || -> Result<reqwest::Response, Error> {
            let builder = match method {
                Method::Post => reqwest::Client::new().post(path.clone()),
                Method::Get => reqwest::Client::new().get(path.clone()),
                Method::Delete => reqwest::Client::new().delete(path.clone()),
            };

            let builder = match body {
                Body::None => builder,
                Body::Form(body) => builder
                    .header(
                        http::header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(body.clone()),
                Body::MultipartForm(multi_form) => {
                    let multi_form = multi_form.clone();
                    let mut form = reqwest::multipart::Form::new();
                    for (name, value) in multi_form {
                        match value {
                            BodyField::File(file) => {
                                let file = file.into_file_path();
                                form = form
                                    .file(name, file)
                                    .await
                                    .map_err(|err| Error::Serialize(err.to_string()))?;
                            }
                            BodyField::Text(text) => {
                                form = form.text(name, text);
                            }
                            BodyField::Bytes(bytes) => {
                                form = form.part(name, reqwest::multipart::Part::bytes(bytes));
                            }
                        }
                    }
                    // .header("Content-Type", "application/octet-stream")
                    builder.multipart(form)
                }
            };
            // let builder = match body {
            //     Some(v) => builder.body(
            //         v.to_form()
            //             .map_err(|err| Error::Serialize(err.to_string()))
            //             .inspect_err(|err| error!("client post err {err}"))?,
            //     ),
            //     None => builder,
            // };

            // let builder = match body_type {
            //     Body::None => builder,
            //     Body::Form => builder.header(
            //         http::header::CONTENT_TYPE,
            //         "application/x-www-form-urlencoded",
            //     ),
            //     Body::MultipartForm => {
            //         let form = reqwest::multipart::Form::new()
            //             .
            //             .file("key", file_path)
            //             .await
            //             .unwrap();

            //         0
            //     }
            // };

            let builder = headers
                .into_iter()
                .fold(builder, |builder, (name, val)| builder.header(name, val));

            let res = builder
                .send()
                .await
                .inspect_err(|err| error!("client post err {err}"))
                .map_err(|err| Error::Send(err.to_string()))?;

            // let status_code = res.status();

            Ok(res)
        };

        let res = inner().await.map(|v| ReqwestWrap(v));

        res
    }
    // async fn send(
    // async fn send<TResult, TError>(
    //     &self,
    //     params: SenderParams,
    //     // method: Method,
    //     // path: impl AsRef<str>,
    //     // body_type: BodyType,
    //     // body: Option<Req>,
    //     // ) -> Self::ResponseContainer<R, E, Req>
    // ) -> ResponseContainer<TResult, TError, Self::TResponse>
    // where
    //     TResult: for<'a> serde::Deserialize<'a> + Debug,
    //     TError: for<'a> serde::Deserialize<'a> + Debug + Default,
    //     // Res: Response
    //     //     Req: serde::Serialize + Debug,
    //     // Wrap: ResponseWrap + Debug,
    //     // Wrap: IntoRes<Res>,
    // {
    //     // let origin = self.origin.clone();
    //     // let params = params.clone();
    //     let origin = &self.origin;
    //     let path = &params.path;
    //     let method = &params.method;
    //     let body = &params.body;
    //     // let body_type = &params.body;
    //     let headers = &params.headers;

    //     let path = origin.join(&path).unwrap();
    //     // let path = gen_url(path);
    //     debug!("CLIENT SEND POST\n{}\n{:#?}", path, body);
    //     let inner = async || -> Result<reqwest::Response, Error> {
    //         let builder = match method {
    //             Method::Post => reqwest::Client::new().post(path.clone()),
    //             Method::Get => reqwest::Client::new().get(path.clone()),
    //         };

    //         let builder = match body {
    //             Body::None => builder,
    //             Body::Form(v) => {
    //                 let body = v
    //                     .to_form()
    //                     .map_err(|err| Error::Serialize(err.to_string()))
    //                     .inspect_err(|err| error!("client post err {err}"))?;
    //                 builder
    //                     .header(
    //                         http::header::CONTENT_TYPE,
    //                         "application/x-www-form-urlencoded",
    //                     )
    //                     .body(body)
    //             }
    //             Body::MultipartForm(multi_form) => {
    //                 let multi_form = multi_form.clone();
    //                 let mut form = reqwest::multipart::Form::new();
    //                 for (name, value) in multi_form {
    //                     match value {
    //                         BodyField::File(file) => {
    //                             form = form
    //                                 .file(name, file)
    //                                 .await
    //                                 .map_err(|err| Error::Serialize(err.to_string()))?;
    //                         }
    //                         BodyField::Text(text) => {
    //                             form = form.text(name, text);
    //                         }
    //                         BodyField::Bytes(bytes) => {
    //                             form = form.part(name, reqwest::multipart::Part::bytes(bytes));
    //                         }
    //                     }
    //                 }
    //                 // .header("Content-Type", "application/octet-stream")
    //                 builder.multipart(form)
    //             }
    //         };
    //         // let builder = match body {
    //         //     Some(v) => builder.body(
    //         //         v.to_form()
    //         //             .map_err(|err| Error::Serialize(err.to_string()))
    //         //             .inspect_err(|err| error!("client post err {err}"))?,
    //         //     ),
    //         //     None => builder,
    //         // };

    //         // let builder = match body_type {
    //         //     Body::None => builder,
    //         //     Body::Form => builder.header(
    //         //         http::header::CONTENT_TYPE,
    //         //         "application/x-www-form-urlencoded",
    //         //     ),
    //         //     Body::MultipartForm => {
    //         //         let form = reqwest::multipart::Form::new()
    //         //             .
    //         //             .file("key", file_path)
    //         //             .await
    //         //             .unwrap();

    //         //         0
    //         //     }
    //         // };

    //         let builder = headers
    //             .into_iter()
    //             .fold(builder, |builder, (name, val)| builder.header(name, val));

    //         let res = builder
    //             .send()
    //             .await
    //             .inspect_err(|err| error!("client post err {err}"))?;

    //         // let status_code = res.status();

    //         Ok(res)
    //     };

    //     let res = inner().await.map(|v| ReqwestWrap(v));

    //     ResponseContainer::new(params, res)
    // }
}
