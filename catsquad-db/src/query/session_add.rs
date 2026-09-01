use crate::{Db, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbSession {
    #[sqlx(rename = "session_token")]
    #[sqlx(try_from = "XUuid")]
    pub token: Uuid,
    #[sqlx(rename = "session_user_email")]
    pub user_email: String,
    #[sqlx(rename = "session_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "session_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbSessionAddErr {
    #[error("user {0} not found")]
    UserNotFound(String),

    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn session_define(&self) {
        // TODO maybe DELETE sessions when email changes
        let pool = &self.db;
        let _result = sqlx::raw_sql(
            "
            CREATE TABLE sessions (
                session_token uuid PRIMARY KEY DEFAULT uuidv7(),
                session_user_email varchar NOT NULL references users(user_email) ON UPDATE CASCADE ON DELETE CASCADE,
                session_modified_at timestamp NOT NULL,
                session_created_at timestamp NOT NULL
            );
            CREATE INDEX session_user_email_idx ON sessions (session_user_email);
        ",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    pub async fn session_add(
        &self,
        time: u64,
        email: impl Into<String>,
    ) -> Result<DbSession, DbSessionAddErr> {
        let pool = &self.db;
        let email = email.into();

        let query = "
            INSERT INTO sessions (
                    session_user_email,
                    session_modified_at,
                    session_created_at
                )
                VALUES ( $2, $1, $1 )
                RETURNING session_token
        ";

        debug!("about to run {query}");

        let result = sqlx::query_as(query)
            .bind(XTimestamp(time as i64))
            .bind(&email)
            .fetch_one(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::Database(err))
                if err.is_foreign_key_violation()
                    && err.constraint() == Some("sessions_session_user_email_fkey") =>
            {
                return Err(DbSessionAddErr::UserNotFound(email));
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbSessionAddErr::Db(err));
            }
        };

        let (XUuid(session_token),): (XUuid,) = result;

        let session = DbSession {
            token: session_token,
            user_email: email,
            modified_at: time,
            created_at: time,
        };

        debug!("query {query}\nresults {session:#?}");

        Ok(session)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_session_add() {
    init_log();

    let db = Db::test_db(0, "test_session_add").await;

    let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    let _result = db
        .user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
        .await
        .unwrap();

    let session1 = db.session_add(0, "hey@hey.com").await.unwrap();
    assert_eq!(session1.user_email, "hey@hey.com");

    let session2 = db.session_add(0, "hey@hey.com").await.unwrap();
    assert_eq!(session2.user_email, "hey@hey.com");

    let result = db.session_add(0, "hey2@hey.com").await;
    assert!(matches!(result, Err(DbSessionAddErr::UserNotFound(_))));
}
