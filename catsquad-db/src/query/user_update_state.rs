use crate::{
    Db, DbEmailSentReason, XTimestamp,
    query::user_get_by_session_and_roles::DbUserGetBySessionAndRolesErr,
};
use catsquad_log::prelude::*;
use catsquad_shared::{ROLE_VALUE_PERMISSION_USER_UPDATE_STATE, SessionTokenKind, UserState};

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdateStateErr {
    #[error("user not found")]
    UserNotFound,

    #[error("unauthorized")]
    Unauthorized(String),

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

fn from_user_get_by_session_and_roles_err(
    value: DbUserGetBySessionAndRolesErr,
) -> DbUserUpdateStateErr {
    match value {
        DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie
        | DbUserGetBySessionAndRolesErr::UnauthorizedSuspended
        | DbUserGetBySessionAndRolesErr::UnauthorizedNoSession
        | DbUserGetBySessionAndRolesErr::InvalidPermissionValue(_)
        | DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. } => {
            DbUserUpdateStateErr::Unauthorized(value.to_string())
        }
        DbUserGetBySessionAndRolesErr::Db(err) => DbUserUpdateStateErr::Db(err),
    }
}

impl Db {
    pub async fn user_update_state(
        &self,
        email_sent_callback: impl AsyncFnOnce() -> String,
        time: u64,
        auth_session_token: SessionTokenKind,
        username: impl Into<String>,
        state: UserState,
    ) -> Result<(), DbUserUpdateStateErr> {
        let pool = &self.db;
        let username = username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        let _user = self
            .auth_required(
                &mut *tx,
                auth_session_token,
                &[ROLE_VALUE_PERMISSION_USER_UPDATE_STATE],
            )
            .await
            .map_err(from_user_get_by_session_and_roles_err)?;

        // update user
        let user_email = {
            let query = "UPDATE users SET
                            user_state = $3,
                            user_modified_at = $1
                            WHERE user_username = $2
                            RETURNING user_email
                            ";

            let result = sqlx::query_as(query)
                .bind(XTimestamp(time as i64))
                .bind(&username)
                .bind(state.as_str())
                .fetch_one(pool)
                .await;

            debug!("query {query}\nresults {result:?}");

            let (user_email,): (String,) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbUserUpdateStateErr::UserNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserUpdateStateErr::Db(err));
                }
            };

            user_email
        };

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        // delete sessions
        {
            let query = "DELETE FROM sessions 
                                USING users
                                WHERE session_user_id=user_id
                                    AND user_username = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(username)
                .bind(state.as_str())
                .execute(pool)
                .await;

            let _result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserUpdateStateErr::Db(err));
                }
            };
        }

        // add email_sent
        {
            let body = email_sent_callback().await;
            let _ = self
                .email_sent_add(time, DbEmailSentReason::UserStateChanged, user_email, body)
                .await;
        }

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_update_state() {
    use crate::user_add_with_roles;

    init_log();

    let db = Db::test_db(0, "test_user_update_state").await;

    let (user1, user1_session) = user_add_with_roles(
        &db,
        0,
        10,
        10,
        "hey1",
        "pss",
        "hey1@heyadora.com",
        vec![ROLE_VALUE_PERMISSION_USER_UPDATE_STATE],
    )
    .await;

    let (user2, user2_session) = user_add_with_roles(
        &db,
        1,
        10,
        10,
        "hey2",
        "pss",
        "hey2@heyadora.com",
        vec![] as Vec<String>,
    )
    .await;

    let email_send_fn = async move || "foo".to_string();

    // assert error
    {
        let result = db
            .user_update_state(
                email_send_fn,
                0,
                user2_session,
                "hey2",
                UserState::Suspended,
            )
            .await;
        assert!(matches!(result, Err(DbUserUpdateStateErr::Unauthorized(_))));
    }

    // assert success
    {
        let sessions = db.session_get_all().await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].user_id, user2.id);
        assert_eq!(sessions[1].user_id, user1.id);

        let result = db
            .user_update_state(
                email_send_fn,
                0,
                user1_session,
                "hey2",
                UserState::Suspended,
            )
            .await;

        let user2 = db.user_get_by_username("", "hey2").await.unwrap();
        let sessions = db.session_get_all().await.unwrap();

        assert!(matches!(result, Ok(_)));
        assert_eq!(user2.state, UserState::Suspended);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].user_id, user1.id);
    }
}
