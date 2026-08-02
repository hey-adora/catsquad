use std::{cell::Cell, fmt::Debug, rc::Rc, time::Duration};

use crate::{
    Body, BodyField, Error, Method, Response, ResponseContainer, Sender, SenderParams, UploadStats,
};
use any_spawner::Executor;
use catsquad_log::prelude::*;
use http::{HeaderMap, StatusCode};
use web_sys::{
    Blob, Event, FormData, XmlHttpRequest,
    js_sys::{Function, Promise, Uint8Array, futures::JsFuture},
    wasm_bindgen::{JsCast, JsValue, prelude::Closure},
};

#[derive(Clone, Debug)]
pub struct XMLSender {
    pub origin: url::Url,
}

impl XMLSender {
    pub fn new() -> Self {
        let origin = web_sys::window().unwrap().location().origin().unwrap();
        let origin = url::Url::parse(origin.as_ref()).unwrap();
        Self { origin }
    }
}

#[derive(Debug)]
pub struct XMLReponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub data: Vec<u8>,
}

impl Response for XMLReponse {
    fn get_status(&self) -> http::StatusCode {
        self.status
    }

    fn get_headers(&self) -> HeaderMap {
        self.headers.clone()
    }

    async fn into_bytes(self) -> Result<Vec<u8>, Error> {
        Ok(self.data)
    }
}

impl Sender for XMLSender {
    type TResponse = XMLReponse;
    // type ResponseContainer<
    //     R: for<'a> serde::Deserialize<'a> + Debug,
    //     E: for<'a> serde::Deserialize<'a> + Debug + Default,
    //     Req: serde::Serialize + Debug,
    // > = ResponseContainer<R, E, Req, XMLReponse>;

    async fn send(&self, params: &SenderParams) -> Result<Self::TResponse, Error> {
        let origin = &self.origin;
        let path = &params.path;
        let method = &params.method;
        let body = &params.body;
        let on_progress = params.on_progress.clone();
        let headers = &params.headers;
        let path = origin.join(&path).unwrap();
        let path_str = path.as_str();

        let inner = async || -> Result<XMLReponse, Error> {
            let req = XmlHttpRequest::new().unwrap();
            req.set_response_type(web_sys::XmlHttpRequestResponseType::Arraybuffer);
            let req_clone0 = req.clone();
            let data = JsFuture::from(Promise::new(
                &mut move |resolve: Function, reject: Function| {
                    let resolve_clone1 = resolve.clone();
                    let req_clone1 = req.clone();
                    let req_upload = req.upload().unwrap();

                    // req_upload
                    //     .add_event_listener_with_callback(
                    //         "progress",
                    //         &Closure::<dyn FnMut(_)>::new(move |event: ProgressEvent| {
                    //             // post_files.update(|v| {
                    //             //     let Some(file) = v.get_mut(index) else {
                    //             //         return;
                    //             //     };
                    //             //     file.completed_bytes = event.loaded() as usize;
                    //             // });
                    //             // trace!("uploading... {}/{}", event.loaded(), event.total());
                    //             //
                    //         })
                    //         .into_js_value()
                    //         .unchecked_into(),
                    //     )
                    //     .unwrap();

                    if let Some(f) = on_progress.clone() {
                        // let total_bytes = match body {
                        //     Body::None => 0,
                        //     Body::Form(v) v.as_bytes().len(),
                        // }
                        let upload_stats = UploadStats::new(Duration::from_secs(1).as_nanos());
                        let upload_stats_rc = Rc::new(Cell::new(upload_stats));
                        req.add_event_listener_with_callback(
                            "progress",
                            &Closure::<dyn FnMut(_)>::new(move |event: web_sys::ProgressEvent| {
                                let total = event.total() as u64;
                                let completed = event.loaded() as u64;
                                let mut upload_stats = upload_stats_rc.get();
                                upload_stats.set_total(total);
                                let time = {
                                    use web_sys::js_sys::Date;
                                    let time = Date::new_0();
                                    let time = time.get_time() as u64;
                                    let time = time as u128 * 1000000;
                                    time
                                };
                                upload_stats.update_by_completed_bytes(time, completed);
                                upload_stats_rc.set(upload_stats.clone());
                                (f.borrow_mut())(upload_stats);
                                // let f = &*f;
                                // let mut f = &mut *f;
                                // f.get_mut()

                                // f();
                                // let v = &mut *f;
                                // v();
                                // let f = f.clone();
                                // Executor::spawn_local(async move {
                                //     (f.write().await)();
                                // });
                                // f();
                                // trace!("downloading... {}/{}", event.loaded(), event.total());
                            })
                            .into_js_value()
                            .unchecked_into(),
                        )
                        .unwrap();
                    }

                    req.add_event_listener_with_callback(
                        "loaded",
                        &Closure::<dyn FnMut()>::new(move || {
                            trace!("complete1");
                            // resolve.call1(&JsValue::NULL, &"done".into()).unwrap();
                        })
                        .into_js_value()
                        .unchecked_into(),
                    )
                    .unwrap();

                    req.add_event_listener_with_callback(
                        "readystatechange",
                        &Closure::<dyn FnMut(_)>::new(move |event: Event| {
                            trace!("complete2");
                            if req_clone1.ready_state() == XmlHttpRequest::DONE {
                                let result = req_clone1
                                    .response()
                                    .inspect_err(|err| {
                                        error!(
                                            "somethnig exploded {}",
                                            err.as_string().unwrap_or_default()
                                        )
                                    })
                                    .ok()
                                    // .map(|v| Uint8Array::new(&v).to_vec())
                                    // .flatten()
                                    .unwrap_or_default();

                                resolve.call1(&JsValue::NULL, &result).unwrap();
                            }
                        })
                        .into_js_value()
                        .unchecked_into(),
                    )
                    .unwrap();

                    let method = method.to_string();
                    req.open_with_async(&method, path_str, true).unwrap();

                    match body {
                        Body::None => {
                            let result = req.send();
                            match result {
                                Ok(_) => (),
                                Err(err) => {
                                    error!("xhr setting header failed {:?}", err.as_string());
                                    resolve_clone1
                                        .call1(&JsValue::NULL, &JsValue::NULL)
                                        .unwrap();
                                    return;
                                }
                            }
                        }
                        Body::Form(data) => {
                            // use catsquad_shared::ToForm;
                            // let result = data.to_form();
                            // let data = match result {
                            //     Ok(data) => data,
                            //     Err(err) => {
                            //         error!("xhr serializing input failed {}", err.to_string());
                            //         resolve_clone1
                            //             .call1(&JsValue::NULL, &JsValue::from_str("error"))
                            //             .unwrap();
                            //         return;
                            //     }
                            // };

                            // .inspect_err(|err| {
                            //     error!("xhr serializing input failed {}", err.to_string())
                            // })
                            // .unwrap_or_default();

                            let result = req.set_request_header(
                                "Content-Type",
                                "application/x-www-form-urlencoded",
                            );
                            match result {
                                Ok(_) => (),
                                Err(err) => {
                                    error!("xhr setting header failed {:?}", err.as_string());
                                    resolve_clone1
                                        .call1(&JsValue::NULL, &JsValue::NULL)
                                        .unwrap();
                                    return;
                                }
                            }
                            // .inspect_err(|err| {
                            //     error!("xhr setting header failed {:?}", err.as_string())
                            // });

                            let result = req.send_with_opt_str(Some(&data));
                            match result {
                                Ok(_) => (),
                                Err(err) => {
                                    error!("xhr sending failed {:?}", err.as_string());
                                    resolve_clone1
                                        .call1(&JsValue::NULL, &JsValue::NULL)
                                        .unwrap();
                                    return;
                                }
                            }
                            //     .inspect_err(|err| {
                            //     error!("xhr sending failed {:?}", err.as_string())
                            // });

                            // v.to
                        }
                        Body::MultipartForm(data) => {
                            let form = FormData::new().unwrap();
                            for (name, value) in data {
                                match value {
                                    BodyField::Text(v) => {
                                        form.set_with_str(name, v).unwrap();
                                    }
                                    BodyField::Bytes(v) => {
                                        let blob = Blob::new().unwrap();
                                        form.set_with_blob(name, &blob).unwrap();
                                        todo!("FIX LATER >:[");
                                    }
                                    BodyField::File(v) => {
                                        let v = v.clone().into_web_file();
                                        form.set_with_blob(name, v.unchecked_ref()).unwrap();
                                    }
                                };
                            }

                            req.send_with_opt_form_data(Some(&form)).unwrap();

                            // form.set_with_blob("upload", file.unchecked_ref()).unwrap();

                            // todo!("not implemented yet");
                            //
                        }
                    }
                    // let form = FormData::new().unwrap();
                    // req.send_with_opt_form_data(Some(&form)).unwrap();
                    // form.set_with_blob("upload", file.unchecked_ref()).unwrap();
                    // form.set_with_str("what", "nooooooooo").unwrap();

                    // let method = method.to_string();
                    // let method = match method {
                    //     Method::Post => "POST",
                    //     Method::Get => "GET",
                    // };

                    // req.set_request_header("Content-Type", "application/x-www-form-urlencoded")
                    //     .unwrap();

                    // req.set_request_header("Content-Type", "multipart/form-data")
                    //     .unwrap();

                    // req.set_request_header("Content-Type", "application/octet-stream")
                    //     .unwrap();
                    // req.send_with_opt_str(Some("hello")).unwrap();
                    // req.send_with_opt_blob(Some(&file)).unwrap();
                },
            ))
            .await
            .map(|v| Uint8Array::new(&v).to_vec())
            .map_err(|err| Error::Send(err.as_string().unwrap_or_default()))?;

            let status = req_clone0
                .status()
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.as_u16());
            let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            let headers = req_clone0.get_all_response_headers().unwrap_or_default();
            trace!("HEADERS FROM XML REQUEST:\n{headers}");
            let headers = HeaderMap::new();

            Ok(XMLReponse {
                status,
                headers,
                data,
            })
        };

        let res = inner().await;

        // let path = gen_url(path);
        // debug!("CLIENT SEND POST\n{}\n{:#?}", path, body);
        // let inner = async || -> Result<reqwest::Response, Error> {
        //     let builder = match method {
        //         Method::Post => reqwest::Client::new().post(path.clone()),
        //         Method::Get => reqwest::Client::new().get(path.clone()),
        //     };

        //     let builder = match body {
        //         Body::None => builder,
        //         Body::Form(v) => {
        //             let body = v
        //                 .to_form()
        //                 .map_err(|err| Error::Serialize(err.to_string()))
        //                 .inspect_err(|err| error!("client post err {err}"))?;
        //             builder
        //                 .header(
        //                     http::header::CONTENT_TYPE,
        //                     "application/x-www-form-urlencoded",
        //                 )
        //                 .body(body)
        //         }
        //         Body::MultipartForm(multi_form) => {
        //             let multi_form = multi_form.clone();
        //             let mut form = reqwest::multipart::Form::new();
        //             for (name, value) in multi_form {
        //                 match value {
        //                     BodyMutlipartField::File(file) => {
        //                         form = form
        //                             .file(name, file)
        //                             .await
        //                             .map_err(|err| Error::Serialize(err.to_string()))?;
        //                     }
        //                     BodyMutlipartField::Text(text) => {
        //                         form = form.text(name, text);
        //                     }
        //                     BodyMutlipartField::Bytes(bytes) => {
        //                         form = form.part(name, reqwest::multipart::Part::bytes(bytes));
        //                     }
        //                 }
        //             }
        //             // .header("Content-Type", "application/octet-stream")
        //             builder.multipart(form)
        //         }
        //     };
        //     // let builder = match body {
        //     //     Some(v) => builder.body(
        //     //         v.to_form()
        //     //             .map_err(|err| Error::Serialize(err.to_string()))
        //     //             .inspect_err(|err| error!("client post err {err}"))?,
        //     //     ),
        //     //     None => builder,
        //     // };

        //     // let builder = match body_type {
        //     //     Body::None => builder,
        //     //     Body::Form => builder.header(
        //     //         http::header::CONTENT_TYPE,
        //     //         "application/x-www-form-urlencoded",
        //     //     ),
        //     //     Body::MultipartForm => {
        //     //         let form = reqwest::multipart::Form::new()
        //     //             .
        //     //             .file("key", file_path)
        //     //             .await
        //     //             .unwrap();

        //     //         0
        //     //     }
        //     // };

        //     let builder = headers
        //         .into_iter()
        //         .fold(builder, |builder, (name, val)| builder.header(name, val));

        //     let res = builder
        //         .send()
        //         .await
        //         .inspect_err(|err| error!("client post err {err}"))?;

        //     // let status_code = res.status();

        //     Ok(res)
        // };

        // let res = inner().await;

        res
        // Self::ResponseContainer::new(params, res)
    }
}
