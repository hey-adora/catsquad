use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};

use crate::{
    Db, DbInvite, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils, create_invite_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbInviteGetByKeyErr {
    #[error("invite not found")]
    InviteNotFound,

    #[error("invite already used")]
    InviteAlreadyUsed,

    #[error("invite expired")]
    InviteExpired,

    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn invite_get_by_key(
        &self,
        time: u128,
        invite_key: impl Into<RecordIdKey>,
    ) -> Result<DbInvite, DbInviteGetByKeyErr> {
        let invite_id = create_invite_id(invite_key);
        let query = r#"
                    BEGIN TRANSACTION;
                    LET $invite = SELECT * FROM ONLY $invite_id;
                    if !$invite {
                        THROW "invite not found"
                    };
                    if $invite.used {
                        THROW "invite already used"
                    };
                    if $invite.expires < $time {
                        THROW "invite expired"
                    };
                    RETURN $invite;
                    COMMIT TRANSACTION;
                "#;

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("time", time))
            .bind(("invite_id", invite_id))
            .await
            .check_better(|err| match err {
                err if err.thrown("invite not found") => DbInviteGetByKeyErr::InviteNotFound,
                err if err.thrown("invite already used") => DbInviteGetByKeyErr::InviteAlreadyUsed,
                err if err.thrown("invite expired") => DbInviteGetByKeyErr::InviteExpired,
                err => {
                    error!("unexpected db error {err}");
                    DbInviteGetByKeyErr::Db(err)
                }
            })
            .and_then_take_or(5, DbInviteGetByKeyErr::InviteNotFound)
    }
}

#[tokio::test]
async fn test_invite_get_by_key() {
    init_log();

    let db = Db::mem(0).await;

    // success
    let invite_key = {
        let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
        let invite2 = db.invite_add(0, "hey@hey.com", 5).await.unwrap();
        let invites = db.invite_get_all().await.unwrap();
        let invite_key = invites[0].id.key.to_sql();
        let invite = db.invite_get_by_key(0, invite_key.clone()).await.unwrap();
        assert_eq!(invite.id.key.to_sql(), invite_key);

        invite_key
    };

    // not found
    {
        let result = db.invite_get_by_key(5, "invalid").await;
        assert_eq!(result, Err(DbInviteGetByKeyErr::InviteNotFound));
    }

    // expired
    {
        let result = db.invite_get_by_key(11, invite_key.clone()).await;
        assert_eq!(result, Err(DbInviteGetByKeyErr::InviteExpired));
    }

    // used
    {
        db.user_add(5, "hey", "r4$$ohnGergnn023n", invite_key.clone(), 10, 10)
            .await
            .unwrap();
        let result = db.invite_get_by_key(5, invite_key.clone()).await;
        assert_eq!(result, Err(DbInviteGetByKeyErr::InviteAlreadyUsed));
    }
}
