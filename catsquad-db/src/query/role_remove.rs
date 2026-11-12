use crate::{Db, query::user_get_by_session_and_roles::DbUserGetBySessionAndRolesErr};
use catsquad_log::prelude::*;
use catsquad_shared::{ROLE_VALUE_PERMISSION_ROLE_REMOVE, SessionTokenKind};

#[derive(Debug, thiserror::Error)]
pub enum DbRoleRemoveErr {
    #[error("not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

fn from_user_get_by_session_and_roles_err(value: DbUserGetBySessionAndRolesErr) -> DbRoleRemoveErr {
    match value {
        DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie
        | DbUserGetBySessionAndRolesErr::UnauthorizedSuspended
        | DbUserGetBySessionAndRolesErr::UnauthorizedNoSession
        | DbUserGetBySessionAndRolesErr::InvalidPermissionValue(_)
        | DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. } => {
            DbRoleRemoveErr::Unauthorized(value.to_string())
        }
        DbUserGetBySessionAndRolesErr::Db(err) => DbRoleRemoveErr::Db(err),
    }
}

impl Db {
    pub async fn role_remove(
        &self,
        auth_session_token: SessionTokenKind,
        role_id: i64,
    ) -> Result<(), DbRoleRemoveErr> {
        let pool = &self.db;

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("role_add {err}"))?;

        // check auth
        let _user = self
            .auth_required(
                &mut *tx,
                auth_session_token,
                &[ROLE_VALUE_PERMISSION_ROLE_REMOVE],
            )
            .await
            .map_err(from_user_get_by_session_and_roles_err)?;

        // remove role
        {
            let query = "DELETE FROM roles WHERE role_id = $1";

            let result = sqlx::query(query).bind(role_id).execute(pool).await;

            debug!("query {query}\nresult {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbRoleRemoveErr::Db(err));
                }
            };

            match result.rows_affected() {
                0 => return Err(DbRoleRemoveErr::NotFound),
                1 => (),
                _ => unreachable!("invalid quary, must only update 1 role max"),
            }
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_role_remove() {
    use catsquad_shared::SessionTokenKind;

    init_log();

    let db = Db::test_db(0, "test_role_remove").await;

    let _role_admin = db
        .role_add(0, SessionTokenKind::Root, "admin", vec!["ban"])
        .await
        .unwrap();
    let role_mod = db
        .role_add(1, SessionTokenKind::Root, "mod", vec!["kick"])
        .await
        .unwrap();

    db.role_remove(SessionTokenKind::Root, role_mod.id)
        .await
        .unwrap();

    // TODO test auth better

    let roles = db.role_get_all().await.unwrap();
    assert_eq!(roles[0].name, "admin");
}
