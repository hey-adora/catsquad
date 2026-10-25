use crate::{Db, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

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
    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn role_define(&self) {
        // role_user_username varchar NOT NULL references users(user_username) ON UPDATE CASCADE ON DELETE CASCADE,
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

    pub async fn role_add<T: Into<String>>(
        &self,
        time: u64,
        // user_username: impl Into<String>,
        name: impl Into<String>,
        values: Vec<T>,
    ) -> Result<DbRole, DbRoleAddErr> {
        let pool = &self.db;
        // let user_username = user_username.into();
        let name = name.into();
        let values = values.into_iter().map(|v| v.into()).collect();

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
            .fetch_one(pool)
            .await;

        debug!("query {query}\nresults {result:#?}");

        let (role_id,): (i64,) = match result {
            Ok(v) => v,
            // Err(sqlx::Error::Database(err))
            //     if err.is_unique_violation()
            //         && err.constraint() == Some("role_role_name_and_role_user_username") =>
            // {
            //     return Err(DbRoleAddErr::RoleAlreadyExists {
            //         // username: user_username,
            //         role: name,
            //     });
            // }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbRoleAddErr::Db(err));
            }
        };

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
    init_log();
    let db = Db::test_db(0, "test_role_add").await;

    // register user
    let user = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user = db
            .user_add(0, "hey", "hey", invite1.token.clone(), 10, 10)
            .await
            .unwrap();
        user
    };

    // assert success
    {
        let role_one = db.role_add(0, "one", vec!["foo"]).await.unwrap();

        let role_two = db.role_add(1, "two", vec!["bar"]).await.unwrap();

        let roles = db.role_get_all().await.unwrap();

        assert_eq!(role_one.name, "one");
        assert_eq!(role_one.values, vec!["foo"]);

        assert_eq!(role_two.name, "two");
        assert_eq!(role_two.values, vec!["bar"]);

        assert_eq!(roles.len(), 2);
        assert_eq!(roles[0].name, "two");
        assert_eq!(roles[1].name, "one");
    }
}
