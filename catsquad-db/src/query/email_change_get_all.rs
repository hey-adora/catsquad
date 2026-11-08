use crate::{Db, DbEmailChange};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbEmailChangeGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_change_get_all(&self) -> Result<Vec<DbEmailChange>, DbEmailChangeGetAllErr> {
        let pool = &self.db;
        let query = "SELECT
                                email_change_id,
                                email_change_user_id,
                                user_username as email_change_user_username,
                                email_change_current_email,
                                email_change_current_token,
                                email_change_current_used,
                                email_change_new_email,
                                email_change_new_token,
                                email_change_new_used,
                                email_change_completed,
                                email_change_expires_at,
                                email_change_modified_at,
                                email_change_created_at
                            FROM emails_changes
                            JOIN users ON email_change_user_id=user_id
                            ORDER BY email_change_created_at DESC";

        let result = sqlx::query_as(query).fetch_all(pool).await;

        trace!("query {query} result {result:#?}");

        let posts = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbEmailChangeGetAllErr::Db(err));
            }
        };

        Ok(posts)
    }
}
