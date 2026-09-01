use crate::{Db, DbInvite, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbInviteGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn invite_get_all(&self) -> Result<Vec<DbInvite>, DbInviteGetAllErr> {
        let pool = &self.db;
        let query = "SELECT invite_token, invite_email, invite_used, invite_expires_at, invite_modified_at, invite_created_at FROM invites ORDER BY invite_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbInviteGetAllErr::Db(err));
            }
        };

        let invites = result
            .into_iter()
            .map(|v| {
                let (
                    token,
                    email,
                    is_used,
                    XTimestamp(expires_at),
                    XTimestamp(modified_at),
                    XTimestamp(created_at),
                ): (XUuid, String, bool, XTimestamp, XTimestamp, XTimestamp) = v;

                let token = token.0;
                let expires_at = expires_at as u64;
                let modified_at = modified_at as u64;
                let created_at = created_at as u64;

                let invite = DbInvite {
                    token,
                    email,
                    used: is_used,
                    expires_at,
                    modified_at,
                    created_at,
                };
                invite
            })
            .collect();

        Ok(invites)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_invite_get_all() {
    init_log();

    let db = Db::test_db(0, "test_invite_get_all").await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let invites = db.invite_get_all().await.unwrap();
    assert_eq!(invites.len(), 1);
}
