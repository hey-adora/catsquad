// use catsquad_log::prelude::*;

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct InviteAddRes {
//     pub expires: u128,
// }

// #[derive(thiserror::Error, Debug, Clone, PartialEq)]
// pub enum SendErr {
//     #[error("serialization failed {0}")]
//     Serialize(String),

//     #[error("deserialization failed {0}")]
//     Deserialization(String),

//     #[error("req send failed {0}")]
//     Send(String),
// }

// impl From<reqwest::Error> for SendErr {
//     fn from(value: reqwest::Error) -> Self {
//         match value {
//             err => Self::Send(err.to_string()),
//         }
//     }
// }

// pub trait ToForm {
//     fn to_form(&self) -> Result<String, anyhow::Error>;
// }

// impl<T: serde::Serialize> ToForm for T {
//     fn to_form(&self) -> Result<String, anyhow::Error> {
//         to_form(self)
//     }
// }

// pub fn to_form(data: impl serde::Serialize) -> Result<String, anyhow::Error> {
//     serde_urlencoded::to_string(data).map_err(|v| v.into())
// }

// pub async fn post<Res: for<'a> serde::Deserialize<'a>>(
//     path: impl AsRef<str>,
//     req: impl serde::Serialize,
// ) -> Result<Res, SendErr> {
//     let path = path.as_ref();
//     let body = to_form(req)
//         .map_err(|err| SendErr::Serialize(err.to_string()))
//         .inspect_err(|err| error!("client post err {err}"))?;
//     let res = reqwest::Client::new()
//         .post(path)
//         .body(body)
//         .send()
//         .await
//         .inspect_err(|err| error!("client post err {err}"))?
//         .json::<Res>()
//         .await
//         .inspect_err(|err| error!("client post err {err}"))?;
//     Ok(res)
// }

// pub async fn get<Res: for<'a> serde::Deserialize<'a>>(
//     path: impl AsRef<str>,
// ) -> Result<Res, SendErr> {
//     let path = path.as_ref();
//     let res = reqwest::get(path)
//         .await
//         .inspect_err(|err| error!("client get err {err}"))?
//         .json::<Res>()
//         .await
//         .inspect_err(|err| error!("client get err {err}"))?;
//     Ok(res)
// }
