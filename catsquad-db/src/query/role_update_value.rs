use catsquad_log::prelude::*;
use catsquad_shared::{ROLE_VALUE_PERMISSION_ROLE_UPDATE_VALUE, SessionTokenKind};

use crate::{Db, DbUserGetBySessionAndRolesErr, DbUserGetBySessionAndRolesSimpleErr, XTimestamp};

#[derive(Debug, thiserror::Error)]
pub enum DbRoleUpdateValueErr {
    #[error("duplicate role values")]
    DuplicateValue,

    #[error("role not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized(String),

    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

fn from_user_get_by_session_and_roles_err(
    value: DbUserGetBySessionAndRolesErr,
) -> DbRoleUpdateValueErr {
    match DbUserGetBySessionAndRolesSimpleErr::from(value) {
        DbUserGetBySessionAndRolesSimpleErr::Unauthorized(err) => {
            DbRoleUpdateValueErr::Unauthorized(err)
        }
        DbUserGetBySessionAndRolesSimpleErr::Db(err) => DbRoleUpdateValueErr::Db(err),
    }
}

impl Db {
    pub async fn role_update_value<T: Into<String>>(
        &self,
        auth_session_token: SessionTokenKind,
        time: u64,
        role_id: i64,
        values: Vec<T>,
    ) -> Result<(), DbRoleUpdateValueErr> {
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
                &[ROLE_VALUE_PERMISSION_ROLE_UPDATE_VALUE],
            )
            .await
            .map_err(from_user_get_by_session_and_roles_err)?;

        let mut values = values
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<String>>();

        // check if duplicate values
        {
            let len_before = values.len();
            values.sort();
            values.dedup();
            let len_after = values.len();
            if len_before != len_after {
                return Err(DbRoleUpdateValueErr::DuplicateValue);
            }
        }

        let query = "
            UPDATE roles SET
                    role_values = $1,
                    role_modified_at = $2
                WHERE role_id = $3
        ";

        let result = sqlx::query(query)
            .bind(&values)
            .bind(XTimestamp(time as i64))
            .bind(role_id)
            .execute(&mut *tx)
            .await;

        debug!("query {query}\nresults {result:#?}");

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbRoleUpdateValueErr::DuplicateValue);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbRoleUpdateValueErr::Db(err));
            }
        };

        match result.rows_affected() {
            0 => return Err(DbRoleUpdateValueErr::NotFound),
            1 => (),
            _ => unreachable!("quary is borked, should only update 1 max"),
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        Ok(())
    }
}
#[cfg(test)]
#[tokio::test]
async fn test_role_update_value() {
    use crate::{user_add_for_testing, user_add_with_roles};

    init_log();
    let db = Db::test_db(0, "test_role_update_value").await;

    // register user
    let _user = user_add_for_testing(&db, 0, "hey", "pss", "hey@heyadora.com", 10, 10).await;
    let token = db.session_add(0, "hey@heyadora.com").await.unwrap().token;

    // setup
    let role_one = {
        use catsquad_shared::SessionTokenKind;

        let role_one = db
            .role_add(0, SessionTokenKind::Root, "one", vec!["foo"])
            .await
            .unwrap();

        role_one
    };

    // asser error
    {
        let result = db
            .role_update_value(SessionTokenKind::Root, 1, 0, vec!["bar"])
            .await;
        assert!(matches!(result, Err(DbRoleUpdateValueErr::NotFound)));

        let role_two = db
            .role_update_value(
                SessionTokenKind::Root,
                1,
                role_one.id,
                vec!["bar", "bar", "bar"],
            )
            .await;
        assert!(matches!(
            role_two,
            Err(DbRoleUpdateValueErr::DuplicateValue)
        ));
    }

    // asser unauthorized
    {
        let result = db
            .role_update_value(SessionTokenKind::None, 1, 0, vec!["bar"])
            .await;
        assert!(matches!(result, Err(DbRoleUpdateValueErr::Unauthorized(_))));

        let result = db
            .role_update_value(SessionTokenKind::User(token), 1, 0, vec!["bar"])
            .await;
        assert!(matches!(result, Err(DbRoleUpdateValueErr::Unauthorized(_))));
    }

    // assert success
    {
        use catsquad_shared::RoleValue;

        db.role_update_value(SessionTokenKind::Root, 0, role_one.id, vec!["bar", "foo"])
            .await
            .unwrap();

        let roles = db.role_get_all().await.unwrap();

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "one");
        assert_eq!(roles[0].values, vec!["bar", "foo"]);

        // give user perms
        {
            db.role_add(
                1,
                SessionTokenKind::Root,
                "mod",
                vec![RoleValue::PermissionRoleUpdateValue],
            )
            .await
            .unwrap();
            db.user_update_role_add(0, "hey", "mod").await.unwrap();
        }

        db.role_update_value(
            SessionTokenKind::User(token),
            0,
            role_one.id,
            vec!["bar", "foo"],
        )
        .await
        .unwrap();

        let roles = db.role_get_all().await.unwrap();

        assert_eq!(roles.len(), 2);
        assert_eq!(roles[0].name, "mod");
        assert_eq!(roles[1].name, "one");
        assert_eq!(roles[1].values, vec!["bar", "foo"]);
    }
}
