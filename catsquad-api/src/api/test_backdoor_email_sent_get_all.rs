use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use catsquad_db::DbEmailSent;
use catsquad_shared::{EmailSentRes, TestBackdoorEmailSentGetAllErr};

use crate::{api::user_add::from_db_user_sensitive, state::AppState};

fn from_db_email_sent(value: DbEmailSent) -> EmailSentRes {
    EmailSentRes {
        body: value.body,
        to_email: value.to_email,
        reason: value.reason,
        created_at: value.created_at,
    }
}

pub async fn test_backdoor_email_sent_get_all(State(app): State<AppState>) -> impl IntoResponse {
    let emails = app
        .db
        .email_sent_get_all()
        .await
        .unwrap()
        .into_iter()
        .map(from_db_email_sent)
        .collect();
    let result = Ok::<Vec<EmailSentRes>, TestBackdoorEmailSentGetAllErr>(emails);

    (StatusCode::OK, Json(result))
}
