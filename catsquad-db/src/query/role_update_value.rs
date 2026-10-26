use catsquad_log::prelude::*;

use crate::{Db, XTimestamp};

#[derive(Debug, thiserror::Error)]
pub enum DbRoleUpdateValueErr {
    #[error("duplicate role values")]
    DuplicateValue,

    #[error("role not found")]
    NotFound,

    #[error("db error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn role_update_value<T: Into<String>>(
        &self,
        time: u64,
        role_id: i64,
        values: Vec<T>,
    ) -> Result<(), DbRoleUpdateValueErr> {
        let pool = &self.db;
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
            .execute(pool)
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

        Ok(())
    }
}
#[cfg(test)]
#[tokio::test]
async fn test_role_update_value() {
    init_log();
    let db = Db::test_db(0, "test_role_update_value").await;

    // register user
    let _user = {
        let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        let user = db
            .user_add(0, "hey", "hey", invite1.token.clone(), 10, 10)
            .await
            .unwrap();
        user
    };

    // assert success
    let role_one = {
        let role_one = db.role_add(0, "one", vec!["foo"]).await.unwrap();
        db.role_update_value(0, role_one.id, vec!["bar", "foo"])
            .await
            .unwrap();

        let roles = db.role_get_all().await.unwrap();

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "one");
        assert_eq!(roles[0].values, vec!["bar", "foo"]);

        role_one
    };

    // asser error
    {
        let result = db.role_update_value(1, 0, vec!["bar"]).await;
        assert!(matches!(result, Err(DbRoleUpdateValueErr::NotFound)));

        let role_two = db
            .role_update_value(1, role_one.id, vec!["bar", "bar", "bar"])
            .await;
        assert!(matches!(
            role_two,
            Err(DbRoleUpdateValueErr::DuplicateValue)
        ));
    }
}
