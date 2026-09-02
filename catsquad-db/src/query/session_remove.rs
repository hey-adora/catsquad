use crate::{Db, DbUser, Uuid, XUuid};
use catsquad_log::prelude::*;
use std::fmt::Display;

// TODO maybe add not found again

#[derive(Debug, thiserror::Error)]
pub enum DbSessionRemoveErr {
    // #[error("not found")]
    // NotFound,
    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn session_remove(&self, session_token: Uuid) -> Result<(), DbSessionRemoveErr> {
        let pool = &self.db;

        let query = "DELETE FROM sessions WHERE session_token = $1";

        debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(XUuid(session_token))
            .execute(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            // Err(sqlx::Error::RowNotFound) => {
            //     // DOESNT GET TRIGGERED ON EXECUTE
            // }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbSessionRemoveErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_session_remove() {
    init_log();

    let db = Db::test_db(0, "test_session_remove").await;

    let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    let _result = db
        .user_add(0, "hey", "hey", invite.token, 10, 10)
        .await
        .unwrap();
    let session = db.session_add(0, "hey@hey.com").await.unwrap();

    db.session_remove(0_u128.to_be_bytes()).await.unwrap();
    // let result = db.session_remove(0_u128.to_be_bytes()).await;
    // assert!(matches!(result, Err(DbSessionRemoveErr::NotFound)));

    let result = db.session_get_by_token(session.token).await;
    assert!(result.is_ok());

    db.session_remove(session.token.clone()).await.unwrap();

    let result = db.session_get_by_token(session.token).await;
    assert!(result.is_err());
}
