use crate::{Db, DbEmailSent};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbEmailSentGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn email_sent_get_all(&self) -> Result<Vec<DbEmailSent>, DbEmailSentGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM emails_sent ORDER BY email_sent_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let posts = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbEmailSentGetAllErr::Db(err));
            }
        };

        Ok(posts)
        // let query = "SELECT * FROM email_sent ORDER BY created_at DESC;";

        // trace!("about to run {query}");

        // self.db
        //     .query(query)
        //     .await
        //     .check_good(|err| match err {
        //         err => {
        //             error!("unexpected db error {err}");
        //             DbEmailSentGetAllErr::Db(err)
        //         }
        //     })
        //     .and_then_take_all(0)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_email_sent_get_all() {
    init_log();

    let db = Db::test_db(0, "test_email_sent_get_all").await;
    let invite = db
        .email_sent_add(
            0,
            crate::DbEmailSentReason::InviteAdd,
            "hey@heyadora.com",
            "hello",
        )
        .await
        .unwrap();
    let invites = db.email_sent_get_all().await.unwrap();
    assert_eq!(invites.len(), 1);
}
