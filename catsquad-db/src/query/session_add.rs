use crate::{Db, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;
use catsquad_shared::UUID_ZERO;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbSession {
    #[sqlx(rename = "session_token")]
    #[sqlx(try_from = "XUuid")]
    pub token: Uuid,
    #[sqlx(rename = "session_user_id")]
    pub user_id: i64,
    #[sqlx(rename = "session_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "session_created_at")]
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
                session_user_id int8 NOT NULL references users(user_id) ON DELETE CASCADE,
                session_modified_at timestamp NOT NULL,
                session_created_at timestamp NOT NULL
            );
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

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        let query = "
            INSERT INTO sessions (
                    session_user_id,
                    session_modified_at,
                    session_created_at
                )
                SELECT
                    user_id, 
                    $2,
                    $2
                    FROM users
                    WHERE user_email = $1
                RETURNING session_token, session_user_id
        ";

        let result = sqlx::query_as(query)
            .bind(&email)
            .bind(XTimestamp(time as i64))
            .fetch_one(&mut *tx)
            .await;

        debug!("query {query}, result {result:#?}");

        let (XUuid(session_token), session_user_id): (XUuid, i64) = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbSessionAddErr::UserNotFound(email));
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbSessionAddErr::Db(err));
            }
        };

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        let session = DbSession {
            token: session_token,
            user_id: session_user_id,
            modified_at: time,
            created_at: time,
        };

        Ok(session)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_session_add() {
    init_log();

    let db = Db::test_db(0, "test_session_add").await;

    let invite = db.invite_add(0, "hey@hey.com", 1).await.unwrap();
    let user1 = db
        .user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
        .await
        .unwrap();

    let session1 = db.session_add(0, "hey@hey.com").await.unwrap();
    assert_eq!(session1.user_id, user1.id);

    let session2 = db.session_add(0, "hey@hey.com").await.unwrap();
    assert_eq!(session2.user_id, user1.id);

    let result = db.session_add(0, "hey2@hey.com").await;
    assert!(matches!(result, Err(DbSessionAddErr::UserNotFound(_))));
}
