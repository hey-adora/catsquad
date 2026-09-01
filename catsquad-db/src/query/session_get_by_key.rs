use crate::{Db, Uuid, XUuid, query::session_add::DbSession};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbSessionGetByKeyErr {
    #[error("session not found")]
    NotFound,

    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn session_get_by_token(
        &self,
        session_token: Uuid,
    ) -> Result<DbSession, DbSessionGetByKeyErr> {
        let pool = &self.db;

        let query = "SELECT * FROM sessions WHERE session_token = $1";

        debug!("about to run {query}");

        let result = sqlx::query_as(query)
            .bind(XUuid(session_token))
            .fetch_one(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbSessionGetByKeyErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbSessionGetByKeyErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        Ok(result)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_session_get_by_key() {
    init_log();

    let db = Db::test_db(0, "test_session_get_by_key").await;

    let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    let _result = db
        .user_add(0, "hey", "hey", invite.token, 10, 10)
        .await
        .unwrap();

    let session = db.session_add(0, "hey@hey.com").await.unwrap();
    assert_eq!(session.user_email, "hey@hey.com");

    let _result = db.session_get_by_token(session.token).await.unwrap();
    let result = db.session_get_by_token(0_u128.to_be_bytes()).await;
    assert!(matches!(result, Err(DbSessionGetByKeyErr::NotFound)));
}
