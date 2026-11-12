use crate::{
    Db, Uuid, XTimestamp, XUuid,
    query::user_get_by_session_and_roles::DbUserGetBySessionAndRolesErr,
};
use catsquad_log::prelude::*;
use catsquad_shared::{
    ROLE_VALUE_PERMISSION_ROLE_ADD, RoleValue, SessionTokenKind, contains_roles,
};

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbRole {
    #[sqlx(rename = "role_id")]
    pub id: i64,
    #[sqlx(rename = "role_name")]
    pub name: String,
    #[sqlx(rename = "role_values")]
    pub values: Vec<String>,
    #[sqlx(rename = "role_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "role_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbRoleAddErr {
    #[error("duplicate role values")]
    DuplicateValue,

    #[error("unauthorized")]
    Unauthorized(String),

    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

fn from_user_get_by_session_and_roles_err(value: DbUserGetBySessionAndRolesErr) -> DbRoleAddErr {
    match value {
        DbUserGetBySessionAndRolesErr::UnauthorizedNoCookie
        | DbUserGetBySessionAndRolesErr::UnauthorizedSuspended
        | DbUserGetBySessionAndRolesErr::UnauthorizedNoSession
        | DbUserGetBySessionAndRolesErr::InvalidPermissionValue(_)
        | DbUserGetBySessionAndRolesErr::UnauthorizedMissingRoles { .. } => {
            DbRoleAddErr::Unauthorized(value.to_string())
        }
        DbUserGetBySessionAndRolesErr::Db(err) => DbRoleAddErr::Db(err),
    }
}

impl Db {
    pub async fn role_define(&self) {
        // DbUserGetBySessionAndRolesErr;
        // CREATE UNIQUE INDEX role_role_name_and_role_user_username ON roles (role_user_username, role_name);
        let pool = &self.db;
        let _result = sqlx::raw_sql(
            "
            CREATE TABLE roles (
                role_id int8 PRIMARY KEY generated always as identity,
                role_name varchar NOT NULL,
                role_values varchar[] DEFAULT array[]::varchar[],
                role_modified_at timestamp NOT NULL,
                role_created_at timestamp NOT NULL
            );
        ",
        )
        .execute(pool)
        .await
        .unwrap();
    }
    pub async fn role_add<TValue>(
        &self,
        time: u64,
        auth_session_token: SessionTokenKind,
        name: impl Into<String>,
        values: Vec<TValue>,
    ) -> Result<DbRole, DbRoleAddErr>
    where
        TValue: Into<String>,
    {
        let pool = &self.db;
        let name = name.into();
        let mut values = values
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<String>>();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("role_add {err}"))?;

        // check auth
        let _user = self
            .auth_required(
                &mut *tx,
                auth_session_token,
                &[ROLE_VALUE_PERMISSION_ROLE_ADD],
            )
            .await
            .map_err(from_user_get_by_session_and_roles_err)?;

        // check if duplicate values
        {
            let len_before = values.len();
            values.sort();
            values.dedup();
            let len_after = values.len();
            if len_before != len_after {
                return Err(DbRoleAddErr::DuplicateValue);
            }
        }

        let query = "
            INSERT INTO roles (
                    role_name,
                    role_values,
                    role_modified_at,
                    role_created_at
                )
                VALUES ( $1, $2, $3, $3)
                RETURNING role_id
        ";

        let result = sqlx::query_as(query)
            .bind(&name)
            .bind(&values)
            .bind(XTimestamp(time as i64))
            .fetch_one(&mut *tx)
            .await;

        debug!("query {query}\nresults {result:#?}");

        let (role_id,): (i64,) = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbRoleAddErr::Db(err));
            }
        };

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        let role = DbRole {
            id: role_id,
            name,
            values,
            modified_at: time,
            created_at: time,
        };

        Ok(role)
    }
}
#[cfg(test)]
#[tokio::test]
async fn test_role_add() {
    use catsquad_shared::ROLE_VALUE_PERMISSION_ROLE_REMOVE;

    use crate::user_add_with_roles;

    init_log();
    let db = Db::test_db(0, "test_role_add").await;

    // register users

    let (user1, user1_session) = user_add_with_roles(
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

    let (user2, user2_session) = user_add_with_roles(
        &db,
        1,
        10,
        10,
        "hey2",
        "pss",
        "hey2@heyadora.com",
        vec![ROLE_VALUE_PERMISSION_ROLE_REMOVE],
    )
    .await;

    // assert error - auth
    {
        let result = db.role_add(0, user2_session, "one", vec!["foo"]).await;
        assert!(matches!(result, Err(DbRoleAddErr::Unauthorized(_))));
    }

    // assert success
    {
        let role_one = db
            .role_add(2, user1_session, "one", vec!["foo"])
            .await
            .unwrap();

        let role_two = db
            .role_add(3, user1_session, "two", vec!["bar"])
            .await
            .unwrap();

        let roles = db.role_get_all().await.unwrap();

        assert_eq!(role_one.name, "one");
        assert_eq!(role_one.values, vec!["foo"]);

        assert_eq!(role_two.name, "two");
        assert_eq!(role_two.values, vec!["bar"]);

        assert_eq!(roles.len(), 4);
        assert_eq!(roles[0].name, "two");
        assert_eq!(roles[1].name, "one");
        assert_eq!(roles[2].name, "hey2_role");
        assert_eq!(roles[3].name, "hey_role");
    }

    // asser error
    {
        let role_two = db
            .role_add(1, user1_session, "two", vec!["bar", "bar"])
            .await;
        assert!(matches!(role_two, Err(DbRoleAddErr::DuplicateValue)));
    }
}
