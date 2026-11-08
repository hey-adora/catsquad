use crate::{Db, DbEmailChange, Uuid, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbEmailChangeGetByIdErr {
    #[error("email change not found")]
    EmailChangeNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("already used")]
    AlreadyUsed,

    #[error("expired")]
    Expired,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_change_get_by_id(
        &self,
        time: u64,
        user_username: impl Into<String>,
        email_change_id: Uuid,
    ) -> Result<DbEmailChange, DbEmailChangeGetByIdErr> {
        let pool = &self.db;
        let user_username = user_username.into();

        let query = "
                SELECT
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
                WHERE email_change_id = $1";

        let result = sqlx::query_as(query)
            .bind(XUuid(email_change_id))
            .fetch_one(pool)
            .await;

        debug!("query {query} result {result:#?}");

        let email_change: DbEmailChange = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbEmailChangeGetByIdErr::EmailChangeNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbEmailChangeGetByIdErr::Db(err));
            }
        };

        if email_change.user_username != user_username {
            return Err(DbEmailChangeGetByIdErr::Unauthorized);
        }

        if email_change.completed {
            return Err(DbEmailChangeGetByIdErr::AlreadyUsed);
        }

        if email_change.expires_at < time {
            return Err(DbEmailChangeGetByIdErr::Expired);
        }

        Ok(email_change)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_change_get_by_id() {
    init_log();

    let db = Db::test_db(0, "test_email_change_get_by_id").await;

    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let user = db
        .user_add(0, "hey", "r4$$ohnGergnn023n", invite.token, 10, 10)
        .await
        .unwrap();
    let email_change = db
        .email_change_add(0, user.username.clone(), 10)
        .await
        .unwrap();
    let _email_change = db
        .email_change_get_by_id(0, user.username.clone(), email_change.id)
        .await
        .unwrap();
}
