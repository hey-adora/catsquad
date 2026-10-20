use crate::{Db, DbTokenLabel, XTimestamp, XUuid};
use catsquad_log::prelude::*;
use catsquad_shared::Uuid;
use sqlx::PgConnection;

#[derive(Debug, thiserror::Error)]
pub enum DbTokenUpdateConfirmErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("expired")]
    Expired,

    #[error("already used")]
    AlreadyUsed,

    #[error("token key not found")]
    TokenNotFound,
}

impl Db {
    pub async fn token_update_confirm_tx(
        &self,
        tx: &mut PgConnection,
        time: u64,
        token_id: Uuid,
        label: DbTokenLabel,
    ) -> Result<String, DbTokenUpdateConfirmErr> {
        // update token
        let user_username = {
            let query = "UPDATE tokens SET
                    token_used = TRUE,
                    token_modified_at = $1
                    WHERE token_id = $2 AND token_label = $3
                    RETURNING token_user_username, OLD.token_used, token_expires_at";

            let result = sqlx::query_as(query)
                .bind(XTimestamp(time as i64))
                .bind(XUuid(token_id))
                .bind(label.to_string())
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query} result {result:#?}");

            let (user_username, used, XTimestamp(expires_at)): (String, bool, XTimestamp) =
                match result {
                    Ok(v) => v,
                    Err(sqlx::Error::RowNotFound) => {
                        return Err(DbTokenUpdateConfirmErr::TokenNotFound);
                    }
                    Err(err) => {
                        error!("unexpected db error {err}");
                        return Err(DbTokenUpdateConfirmErr::from(err));
                    }
                };

            let expires_at = expires_at as u64;

            if expires_at < time {
                return Err(DbTokenUpdateConfirmErr::Expired);
            }

            if used {
                return Err(DbTokenUpdateConfirmErr::AlreadyUsed);
            }

            user_username
        };

        Ok(user_username)
    }
}
