use crate::{
    Db, DbEmailSentReason, XTimestamp,
    query::user_get_by_session_and_roles::DbUserGetBySessionAndRolesErr,
};
use catsquad_log::prelude::*;
use catsquad_shared::{ROLE_VALUE_PERMISSION_USER_UPDATE_STATE, SessionTokenKind, UserState};

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdateStateErr {
    #[error("unauthorized")]
    Unauthorized(String),

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbUserUpdateState {
    #[sqlx(rename = "user_username")]
    pub username: String,
    #[sqlx(rename = "user_email")]
    pub email: String,
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
        mut email_sent_callback: impl AsyncFnMut(DbUserUpdateState) -> String,
        time: u64,
        auth_session_token: SessionTokenKind,
        usernames: Vec<String>,
        state: UserState,
    ) -> Result<Vec<DbUserUpdateState>, DbUserUpdateStateErr> {
        let pool = &self.db;

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
        let output_data = {
            let query = "UPDATE users SET
                            user_state = $3,
                            user_modified_at = $1
                            WHERE user_username = ANY($2)
                            RETURNING user_username, user_email
                            ";

            let result = sqlx::query_as(query)
                .bind(XTimestamp(time as i64))
                .bind(&usernames)
                .bind(state.as_str())
                .fetch_all(pool)
                .await;

            debug!("query {query}\nresults {result:?}");

            let data_for_email: Vec<DbUserUpdateState> = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserUpdateStateErr::Db(err));
                }
            };

            data_for_email
        };

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        // delete sessions
        {
            let query = "DELETE FROM sessions 
                                USING users
                                WHERE session_user_id=user_id
                                    AND user_username = ANY($1)";

            let result = sqlx::query(query).bind(usernames).execute(pool).await;

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
            for data in output_data.clone() {
                let to_email = data.email.clone();
                let body = email_sent_callback(data).await;
                let _ = self
                    .email_sent_add(time, DbEmailSentReason::UserStateChanged, to_email, body)
                    .await;
            }
        }

        Ok(output_data)
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

    let (user3, user3_session) = user_add_with_roles(
        &db,
        2,
        10,
        10,
        "hey3",
        "pss",
        "hey3@heyadora.com",
        vec![] as Vec<String>,
    )
    .await;

    let email_send_fn = async move |_| "foo".to_string();

    // TODO maybe better auth check test

    // assert error
    {
        let result = db
            .user_update_state(
                email_send_fn,
                0,
                user2_session,
                vec!["hey2".to_string(), "hey3".to_string()],
                UserState::Suspended,
            )
            .await;
        assert!(matches!(result, Err(DbUserUpdateStateErr::Unauthorized(_))));
    }

    // assert success
    {
        let sessions = db.session_get_all().await.unwrap();
        assert_eq!(sessions.len(), 3);
        assert_eq!(sessions[0].user_id, user3.id);
        assert_eq!(sessions[1].user_id, user2.id);
        assert_eq!(sessions[2].user_id, user1.id);

        let result = db
            .user_update_state(
                email_send_fn,
                0,
                user1_session,
                vec!["hey2".to_string(), "hey3".to_string()],
                UserState::Suspended,
            )
            .await;

        let user1 = db.user_get_by_username("", "hey1").await.unwrap();
        let user2 = db.user_get_by_username("", "hey2").await.unwrap();
        let user3 = db.user_get_by_username("", "hey3").await.unwrap();
        let sessions = db.session_get_all().await.unwrap();

        assert!(matches!(result, Ok(_)));
        assert_eq!(user1.state, UserState::Active);
        assert_eq!(user2.state, UserState::Suspended);
        assert_eq!(user3.state, UserState::Suspended);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].user_id, user1.id);
    }
}
