use axum::{Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::{DbInvite, DbInviteAddErr, id_to_string};
use catsquad_log::prelude::*;
use catsquad_shared::{
    InviteAddErr, InviteAddReq, InviteRes, link_absolute_reg_finish, validate_email,
};

use crate::state::AppState;

fn from_db_invite(value: DbInvite) -> InviteRes {
    InviteRes {
        expires: value.expires,
    }
}

fn status_code(result: &Result<InviteRes, InviteAddErr>) -> StatusCode {
    match result {
        Ok(_) => StatusCode::OK,
        Err(InviteAddErr::InvalidEmail(_)) => StatusCode::BAD_REQUEST,
        Err(InviteAddErr::BadRequest(_)) => StatusCode::BAD_REQUEST,
        Err(InviteAddErr::InternalServer) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn send_email_invite(address: impl AsRef<str>, token: impl AsRef<str>) -> String {
    let link = link_absolute_reg_finish(address, token);
    debug!("EMAIL SENT {link}");
    link
}

pub async fn invite_add(
    State(app): State<AppState>,
    Form(req): Form<InviteAddReq>,
) -> impl IntoResponse {
    let time = app.get_time().await;
    let invite_expiration = app.get_invite_expiration().await;
    let inner = async || -> Result<InviteRes, InviteAddErr> {
        let email = req.email.trim().to_lowercase();
        validate_email(&email).map_err(|err| InviteAddErr::InvalidEmail(err))?;

        let expires = time + invite_expiration;
        let result = app.db.invite_add(time, &email, expires).await;
        let invite = match result {
            Ok(v) => v,
            Err(DbInviteAddErr::EmailIsTaken(_)) => return Ok(InviteRes { expires }),
            Err(DbInviteAddErr::Db(_)) => return Err(InviteAddErr::InternalServer),
        };

        let address = app.get_address().await;
        let email_body = send_email_invite(address, &id_to_string(invite.id.clone()));
        let _ = app
            .db
            .email_sent_add(
                0,
                catsquad_db::DbEmailSentReason::InviteAdd,
                email,
                email_body,
            )
            .await;

        Ok(from_db_invite(invite))
    };

    let result = inner().await;
    let status_code = status_code(&result);

    (status_code, Json(result))
}

#[cfg(test)]
mod test_utils {
    use crate::TestServer;
    use catsquad_db::id_to_string;
    use catsquad_shared as cs;

    impl TestServer {
        pub async fn invite_add(
            &self,
            email: impl Into<String>,
        ) -> Result<cs::InviteRes, cs::InviteAddErr> {
            self.client.invite_add(email).send().await.into_res().await
        }

        pub async fn invite_get_key(&self, email: impl AsRef<str>) -> String {
            let email = email.as_ref();
            id_to_string(
                self.state
                    .db
                    .invite_get_all()
                    .await
                    .unwrap()
                    .into_iter()
                    .find(|v| !v.used && v.email == *email)
                    .unwrap()
                    .id
                    .clone(),
            )
        }
    }
}

// #[cfg(test)]
// mod test_utils {
//     use catsquad_shared::{InviteAddErr, InviteAddReq, InviteRes, LINK_API_INVITE_ADD, ToForm};

//     use crate::TestServer;

//     impl TestServer {
//         pub async fn invite_add(
//             &self,
//             email: impl Into<String>,
//         ) -> Result<InviteRes, InviteAddErr> {
//             let data = InviteAddReq {
//                 email: email.into(),
//             }
//             .to_form()
//             .unwrap();
//             self.post::<Result<InviteRes, InviteAddErr>>(LINK_API_INVITE_ADD, data)
//                 .await
//         }
//     }
// }

// #[tokio::test]
// async fn test_invite_add() {
//     init_log();
//     let server = crate::TestServer::new().await;

//     let result = server.client.api_invite_add("hello").await.into_res().await;
//     // let result = server.invite_add("hello").await;
//     assert!(matches!(result, Err(InviteAddErr::InvalidEmail(_))));

//     let result = server.invite_add("prime@heyadora.com").await;
//     assert!(result.is_ok());
// }
