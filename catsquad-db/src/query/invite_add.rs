use crate::{Db, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub struct DbInvite {
    pub token: Uuid,
    pub email: String,
    pub used: bool,
    pub expires_at: u64,
    pub modified_at: u64,
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbInviteAddErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("account with \"{0}\" email already exists")]
    EmailIsTaken(String),
}

impl Db {
    pub async fn invite_define(&self) {
        let pool = &self.db;
        let _result = sqlx::raw_sql(
            "
            CREATE TABLE invites (
                invite_token uuid PRIMARY KEY DEFAULT uuidv7(),
                invite_email varchar(100) NOT NULL,
                invite_used bool NOT NULL DEFAULT false,
                invite_expires_at timestamp NOT NULL,
                invite_modified_at timestamp NOT NULL,
                invite_created_at timestamp NOT NULL
            );
        ",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    pub async fn invite_add(
        &self,
        time: u64,
        email: impl Into<String>,
        expires: u64,
    ) -> Result<DbInvite, DbInviteAddErr> {
        let pool = &self.db;

        let email = email.into();

        let mut tx = pool.begin().await?;

        let query = "SELECT EXISTS(SELECT 1 FROM users WHERE user_email=$1)";
        trace!("about to run {query}");
        let result = sqlx::query_as(query).bind(&email).fetch_one(&mut *tx).await;

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbInviteAddErr::Db(err));
            }
        };

        let (email_is_taken,): (bool,) = result;

        trace!("query: {query}\nresult: {email_is_taken}");

        if email_is_taken {
            return Err(DbInviteAddErr::EmailIsTaken(email));
        }

        let query = "
            INSERT INTO invites (
                    invite_email,
                    invite_expires_at,
                    invite_modified_at,
                    invite_created_at
                )
                VALUES ( $1, $2, $3, $3 )
                RETURNING invite_token
        ";

        let result = sqlx::query_as(query)
            .bind(&email)
            .bind(XTimestamp(expires as i64))
            .bind(XTimestamp(time as i64))
            .fetch_one(&mut *tx)
            .await;

        trace!("query: {query}\n$1={email},expires={expires},time={time}\nresult: {result:#?}");

        tx.commit().await?;

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbInviteAddErr::Db(err));
            }
        };

        let (XUuid(invite_token),): (XUuid,) = result;

        let db_invite = DbInvite {
            email,
            token: invite_token,
            used: false,
            expires_at: expires,
            modified_at: time,
            created_at: time,
        };
        // trace!("\nquery: {query}\nresult: {db_invite:#?}");

        Ok(db_invite)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_invite_add() {
    use std::time::Duration;

    init_log();

    let db = Db::test_db(0, "test_invite_add").await;

    let time = Duration::from_micros(1).as_micros() as u64;
    let expires_at = Duration::from_secs(1).as_micros() as u64;
    let invite1 = db
        .invite_add(time, "prime@heyadora.com", expires_at)
        .await
        .unwrap();
    trace!("received \n{invite1:#?}");
    assert_eq!(invite1.expires_at, expires_at);
    assert_eq!(invite1.modified_at, time);
    assert_eq!(invite1.created_at, time);

    let invite2 = db.invite_add(0, "prime@heyadora.com", 1).await.unwrap();
    trace!("received \n{invite2:#?}");
    assert_eq!(invite2.expires_at, 1);
    assert_eq!(invite2.modified_at, 0);
    assert_eq!(invite2.created_at, 0);

    let invite3 = db.invite_add(0, "prime@heyadora.com", 0).await.unwrap();
    trace!("received \n{invite3:#?}");
    assert_eq!(invite3.expires_at, 0);
    assert_eq!(invite3.modified_at, 0);
    assert_eq!(invite3.created_at, 0);

    db.user_add(0, "prime", "pss", invite2.token, 10, 5)
        .await
        .unwrap();

    let result = db.invite_add(0, "prime@heyadora.com", 1).await;
    assert!(matches!(result, Err(DbInviteAddErr::EmailIsTaken(_))));
}
