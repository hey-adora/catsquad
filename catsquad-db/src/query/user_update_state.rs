use crate::{Db, XTimestamp, query::user_get_by_session_and_roles::DbUserGetBySessionAndRolesErr};
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
        time: u64,
        auth_session_token: SessionTokenKind,
        user_username: impl Into<String>,
        state: UserState,
    ) -> Result<(), DbUserUpdateStateErr> {
        let pool = &self.db;
        let user_username = user_username.into();

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
        {
            let query = "UPDATE users SET
                            user_state = $3,
                            user_modified_at = $1
                            WHERE user_username = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(user_username)
                .bind(state.as_str())
                .execute(pool)
                .await;

            debug!("query {query}\nresults {result:?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserUpdateStateErr::Db(err));
                }
            };

            let affected = result.rows_affected();

            match affected {
                1 => (),
                0 => return Err(DbUserUpdateStateErr::UserNotFound),
                _ => panic!("quary is wrong, must only modify 1 user"),
            }
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        // delete sessions
        {
            let query = "DELETE FROM sessions 
                                WHERE session_user_id = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(user_username)
                .bind(state.as_str())
                .execute(pool)
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
        0,
        10,
        10,
        "hey2",
        "pss",
        "hey2@heyadora.com",
        vec![] as Vec<String>,
    )
    .await;

    // assert error
    {
        let result = db
            .user_update_state(0, user2_session, "hey2", UserState::Suspended)
            .await;
        assert!(matches!(result, Err(DbUserUpdateStateErr::Unauthorized(_))));
    }

    // assert success
    {
        let result = db
            .user_update_state(0, user1_session, "hey2", UserState::Suspended)
            .await;
        let user2 = db.user_get_by_username("", "hey2").await.unwrap();

        assert!(matches!(result, Ok(_)));
        assert_eq!(user2.state, UserState::Suspended);
    }

    // let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    // let user = db
    //     .user_add(0, "hey", "hey", invite1.token.clone(), 10, 5)
    //     .await
    //     .unwrap();
    // assert_eq!(user.max_storage_bytes, 10);
    // assert_eq!(user.max_storage_per_image_bytes, 5);

    // db.user_update_storage(0, user.username.clone(), 10, 5)
    //     .await
    //     .unwrap();

    // let user = db.user_get_by_username("", "hey").await.unwrap();

    // assert_eq!(user.max_storage_bytes, 10);
    // assert_eq!(user.max_storage_per_image_bytes, 5);
}
