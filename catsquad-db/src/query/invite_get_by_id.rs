use crate::{Db, DbInvite, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbInviteGetByKeyErr {
    #[error("invite not found")]
    InviteNotFound,

    #[error("invite already used")]
    InviteAlreadyUsed,

    #[error("invite expired")]
    InviteExpired,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn invite_get_by_key(
        &self,
        time: u64,
        invite_token: Uuid,
    ) -> Result<DbInvite, DbInviteGetByKeyErr> {
        let pool = &self.db;
        let query = "SELECT invite_email, invite_used, invite_expires_at, invite_modified_at, invite_created_at FROM invites WHERE invite_token = $1";

        let result = sqlx::query_as(query)
            .bind(XUuid(invite_token))
            .fetch_one(pool)
            .await;

        trace!("query {query} results: {result:#?}");

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbInviteGetByKeyErr::InviteNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbInviteGetByKeyErr::Db(err));
            }
        };

        let (
            email,
            is_used,
            XTimestamp(expires_at),
            XTimestamp(modified_at),
            XTimestamp(created_at),
        ): (String, bool, XTimestamp, XTimestamp, XTimestamp) = result;
        let expires_at = expires_at as u64;
        let modified_at = modified_at as u64;
        let created_at = created_at as u64;

        if is_used {
            return Err(DbInviteGetByKeyErr::InviteAlreadyUsed);
        }

        if expires_at < time {
            return Err(DbInviteGetByKeyErr::InviteExpired);
        }

        let invite = DbInvite {
            token: invite_token,
            email,
            used: is_used,
            expires_at,
            modified_at,
            created_at,
        };

        trace!("query: {query}\nresult: {invite:#?}");

        Ok(invite)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_invite_get_by_key() {
    init_log();

    let db = Db::test_db(0, "test_invite_get_by_key").await;

    // success
    let invite_key = {
        let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
        let invite2 = db.invite_add(0, "hey@hey.com", 5).await.unwrap();
        let invites = db.invite_get_all().await.unwrap();
        let invite_token = invites[0].token;
        let invite = db.invite_get_by_key(0, invite_token.clone()).await.unwrap();
        assert_eq!(invite.token, invite_token);

        invite_token
    };

    // not found
    {
        let result = db.invite_get_by_key(5, 0_u128.to_be_bytes()).await;
        assert!(matches!(result, Err(DbInviteGetByKeyErr::InviteNotFound)));
    }

    // expired
    {
        let result = db.invite_get_by_key(11, invite_key.clone()).await;
        assert!(matches!(result, Err(DbInviteGetByKeyErr::InviteExpired)));
    }

    // used
    {
        db.user_add(5, "hey", "r4$$ohnGergnn023n", invite_key.clone(), 10, 10)
            .await
            .unwrap();
        let result = db.invite_get_by_key(5, invite_key.clone()).await;
        assert!(matches!(
            result,
            Err(DbInviteGetByKeyErr::InviteAlreadyUsed)
        ));
    }
}
