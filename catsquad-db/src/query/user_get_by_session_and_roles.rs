use std::fmt::Debug;

use crate::{Db, XUuid, query::role_get_by_role_tx::DbRoleGetTxErr};
use catsquad_log::prelude::*;
use catsquad_shared::{RoleValue, SessionTokenKind, UserState, Uuid, contains_roles};
use sqlx::PgConnection;

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetBySessionAndRolesErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("no cookie found")]
    UnauthorizedNoCookie,

    #[error("no session found")]
    UnauthorizedNoSession,

    #[error("user suspended")]
    UnauthorizedSuspended,

    #[error(
        "missing permissions for user {username}, has {user_permissions:?}, expected {expected_permissions:?}"
    )]
    UnauthorizedMissingRoles {
        username: String,
        user_permissions: Vec<RoleValue>,
        expected_permissions: Vec<RoleValue>,
    },

    #[error("invalid permission value {0}")]
    InvalidPermissionValue(String),
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbUserGetBySessionAndRoles {
    #[sqlx(rename = "user_id")]
    pub id: i64,
    #[sqlx(rename = "user_state")]
    pub state: String,
    #[sqlx(rename = "user_username")]
    pub username: String,
    #[sqlx(rename = "user_roles")]
    pub roles: Vec<i64>,
    #[sqlx(default)]
    pub roles_values: Vec<String>,
}

impl Db {
    /// returns valid user OR throws error
    pub async fn auth_user<TPermissions>(
        &self,
        tx: &mut PgConnection,
        session_token: SessionTokenKind,
        expected_permissions: &[TPermissions],
    ) -> Result<DbUserGetBySessionAndRoles, DbUserGetBySessionAndRolesErr>
    where
        TPermissions: TryInto<RoleValue> + Clone + Debug,
        TPermissions::Error: ToString,
    {
        match session_token {
            SessionTokenKind::User(token) => {
                let user = self
                    .user_get_by_session_and_roles_tx(&mut *tx, token, expected_permissions)
                    .await;

                match user {
                    Ok(v) => Ok(v),
                    Err(err) => Err(err),
                }
            }
            SessionTokenKind::None => Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie),
            SessionTokenKind::Root => Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie),
        }
    }

    /// returns optional user or root OR throws error
    pub async fn auth_required<TPermissions>(
        &self,
        tx: &mut PgConnection,
        session_token: SessionTokenKind,
        expected_permissions: &[TPermissions],
    ) -> Result<Option<DbUserGetBySessionAndRoles>, DbUserGetBySessionAndRolesErr>
    where
        TPermissions: TryInto<RoleValue> + Clone + Debug,
        TPermissions::Error: ToString,
    {
        match session_token {
            SessionTokenKind::User(token) => {
                let user = self
                    .user_get_by_session_and_roles_tx(&mut *tx, token, expected_permissions)
                    .await;

                match user {
                    Ok(v) => Ok(Some(v)),
                    Err(err) => Err(err),
                }
            }
            SessionTokenKind::None => Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie),
            SessionTokenKind::Root => Ok(None),
        }
    }

    /// returns optional user or root, doesn't throws error
    pub async fn auth_optional<TPermissions>(
        &self,
        tx: &mut PgConnection,
        session_token: SessionTokenKind,
        expected_permissions: &[TPermissions],
    ) -> Result<Option<DbUserGetBySessionAndRoles>, DbUserGetBySessionAndRolesErr>
    where
        TPermissions: TryInto<RoleValue> + Clone + Debug,
        TPermissions::Error: ToString,
    {
        match session_token {
            SessionTokenKind::User(token) => {
                let user = self
                    .user_get_by_session_and_roles_tx(&mut *tx, token, expected_permissions)
                    .await;

                match user {
                    Ok(v) => Ok(Some(v)),
                    Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoSession) => Ok(None),
                    Err(DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. }) => Ok(None),
                    Err(err) => Err(err),
                }
            }
            SessionTokenKind::None => Ok(None),
            SessionTokenKind::Root => Ok(None),
        }
    }

    pub async fn user_get_by_session_and_roles_tx<TPermissions>(
        &self,
        tx: &mut PgConnection,
        session_token: Uuid,
        expected_permissions: &[TPermissions],
    ) -> Result<DbUserGetBySessionAndRoles, DbUserGetBySessionAndRolesErr>
    where
        TPermissions: TryInto<RoleValue> + Clone + Debug,
        TPermissions::Error: ToString,
    {
        // get user
        let mut user = {
            let query = "SELECT
                                user_id,
                                user_state,
                                user_username,
                                user_roles
                             FROM users
                             JOIN sessions ON user_id=session_user_id
                             WHERE session_token = $1";

            let result = sqlx::query_as(query)
                .bind(XUuid(session_token))
                .fetch_one(&mut *tx)
                .await;

            trace!("query {query} result {result:#?}");

            let user: DbUserGetBySessionAndRoles = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoSession);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserGetBySessionAndRolesErr::Db(err));
                }
            };

            user
        };

        if user.state == UserState::Suspended {
            return Err(DbUserGetBySessionAndRolesErr::UnauthorizedSuspended);
        }

        // get roles
        {
            let roles_values = self
                .role_get_by_role_ids_tx(&user.roles, &mut *tx)
                .await
                .map_err(|err| match err {
                    DbRoleGetTxErr::Db(err) => DbUserGetBySessionAndRolesErr::Db(err),
                })?;

            user.roles_values = roles_values;

            let user_permissions = user
                .roles_values
                .iter()
                .map(|v| RoleValue::try_from(v.as_str()))
                .collect::<Result<Vec<RoleValue>, _>>()
                .map_err(|err| {
                    DbUserGetBySessionAndRolesErr::InvalidPermissionValue(err.to_string())
                })?;

            let expected_permissions = expected_permissions
                .iter()
                .map(|v| v.clone().try_into()) // TODO maybe add try_from(&T)
                .collect::<Result<Vec<RoleValue>, TPermissions::Error>>()
                .map_err(|err| {
                    DbUserGetBySessionAndRolesErr::InvalidPermissionValue(err.to_string())
                })?;

            let has_permissions = contains_roles(&expected_permissions, &user_permissions);

            if !has_permissions {
                return Err(DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles {
                    username: user.username,
                    user_permissions: user_permissions,
                    expected_permissions: expected_permissions,
                });
            }
        }

        trace!("user_get_by_session_and_roles_tx output {user:#?}");

        Ok(user)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_get_by_session_and_roles() {
    use catsquad_shared::{
        ROLE_VALUE_PERMISSION_ROLE_ADD, ROLE_VALUE_PERMISSION_ROLE_REMOVE, UUID_ZERO,
    };

    use crate::user_add_with_roles;

    init_log();

    let db = Db::test_db(0, "test_user_get_by_session_and_roles").await;

    let (_user1, user1_session) = user_add_with_roles(
        &db,
        0,
        10,
        10,
        "hey",
        "pss",
        "hey@heyadora.com",
        vec![ROLE_VALUE_PERMISSION_ROLE_ADD],
    )
    .await;

    let user1_session = match user1_session {
        SessionTokenKind::User(v) => v,
        _ => panic!("no"),
    };

    let user_get_by_session_and_roles_tx = async |token: Uuid, perms: &[&str]| {
        let mut tx = db.db.begin().await.unwrap();
        let result = db
            .user_get_by_session_and_roles_tx(&mut *tx, token, perms)
            .await;
        tx.commit().await.unwrap();
        result
    };

    let auth_optional = async |token: SessionTokenKind, perms: &[&str]| {
        let mut tx = db.db.begin().await.unwrap();
        let result = db.auth_optional(&mut *tx, token, perms).await;
        tx.commit().await.unwrap();
        result
    };

    let auth_required = async |token: SessionTokenKind, perms: &[&str]| {
        let mut tx = db.db.begin().await.unwrap();
        let result = db.auth_required(&mut *tx, token, perms).await;
        tx.commit().await.unwrap();
        result
    };

    let auth_user = async |token: SessionTokenKind, perms: &[&str]| {
        let mut tx = db.db.begin().await.unwrap();
        let result = db.auth_user(&mut *tx, token, perms).await;
        tx.commit().await.unwrap();
        result
    };

    // assert success
    {
        let result =
            user_get_by_session_and_roles_tx(user1_session, &[ROLE_VALUE_PERMISSION_ROLE_ADD])
                .await;
        assert!(matches!(result, Ok(_)));
    }

    // assert error missing roles
    {
        let result =
            user_get_by_session_and_roles_tx(user1_session, &[ROLE_VALUE_PERMISSION_ROLE_REMOVE])
                .await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. })
        ));
    }

    // assert error no session
    {
        let result =
            user_get_by_session_and_roles_tx(UUID_ZERO, &[ROLE_VALUE_PERMISSION_ROLE_REMOVE]).await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoSession)
        ));
    }

    // assert error invalid permission value
    {
        let result = user_get_by_session_and_roles_tx(user1_session, &["invalid"]).await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::InvalidPermissionValue(_))
        ));
    }

    // assert auth optional
    {
        let result = auth_optional(SessionTokenKind::None, &[]).await;
        assert!(matches!(result, Ok(None)));

        let result = auth_optional(SessionTokenKind::Root, &[]).await;
        assert!(matches!(result, Ok(None)));

        let result = auth_optional(SessionTokenKind::User(UUID_ZERO), &[]).await;
        assert!(matches!(result, Ok(None)));

        let result = auth_optional(SessionTokenKind::User(user1_session), &[]).await;
        assert!(matches!(result, Ok(Some(..))));

        let result = auth_optional(
            SessionTokenKind::User(user1_session),
            &[ROLE_VALUE_PERMISSION_ROLE_REMOVE],
        )
        .await;
        assert!(matches!(result, Ok(None)));

        let result = auth_optional(
            SessionTokenKind::User(user1_session),
            &[
                ROLE_VALUE_PERMISSION_ROLE_REMOVE,
                ROLE_VALUE_PERMISSION_ROLE_ADD,
            ],
        )
        .await;
        assert!(matches!(result, Ok(None)));

        let result = auth_optional(
            SessionTokenKind::User(user1_session),
            &[ROLE_VALUE_PERMISSION_ROLE_ADD],
        )
        .await;
        assert!(matches!(result, Ok(Some(..))));

        let result = auth_optional(SessionTokenKind::User(user1_session), &["invalid"]).await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::InvalidPermissionValue(_))
        ));
    }

    // assert auth required
    {
        let result = auth_required(SessionTokenKind::None, &[]).await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie)
        ));

        let result = auth_required(SessionTokenKind::Root, &[]).await;
        assert!(matches!(result, Ok(None)));

        let result = auth_required(SessionTokenKind::User(UUID_ZERO), &[]).await;
        assert!(matches!(result, Err(_)));

        let result = auth_required(SessionTokenKind::User(user1_session), &[]).await;
        assert!(matches!(result, Ok(Some(..))));

        let result = auth_required(
            SessionTokenKind::User(user1_session),
            &[ROLE_VALUE_PERMISSION_ROLE_REMOVE],
        )
        .await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. })
        ));

        let result = auth_required(
            SessionTokenKind::User(user1_session),
            &[
                ROLE_VALUE_PERMISSION_ROLE_REMOVE,
                ROLE_VALUE_PERMISSION_ROLE_ADD,
            ],
        )
        .await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. })
        ));

        let result = auth_required(
            SessionTokenKind::User(user1_session),
            &[ROLE_VALUE_PERMISSION_ROLE_ADD],
        )
        .await;
        assert!(matches!(result, Ok(Some(..))));
    }

    // assert auth optional
    {
        let result = auth_user(SessionTokenKind::None, &[]).await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie)
        ));

        let result = auth_user(SessionTokenKind::Root, &[]).await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie)
        ));

        let result = auth_user(SessionTokenKind::User(UUID_ZERO), &[]).await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedNoSession)
        ));

        let result = auth_user(SessionTokenKind::User(user1_session), &[]).await;
        assert!(matches!(result, Ok(..)));

        let result = auth_user(
            SessionTokenKind::User(user1_session),
            &[ROLE_VALUE_PERMISSION_ROLE_REMOVE],
        )
        .await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. })
        ));

        let result = auth_user(
            SessionTokenKind::User(user1_session),
            &[
                ROLE_VALUE_PERMISSION_ROLE_REMOVE,
                ROLE_VALUE_PERMISSION_ROLE_ADD,
            ],
        )
        .await;
        assert!(matches!(
            result,
            Err(DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. })
        ));

        let result = auth_user(
            SessionTokenKind::User(user1_session),
            &[ROLE_VALUE_PERMISSION_ROLE_ADD],
        )
        .await;
        assert!(matches!(result, Ok(_)));
    }
}
