use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserUpdateRoleRemoveErr {
    #[error("role {name} not found")]
    RoleNotFound { name: String },

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn user_update_role_remove(
        &self,
        time: u64,
        user_username: impl Into<String>,
        role_name: impl Into<String>,
    ) -> Result<(), DbUserUpdateRoleRemoveErr> {
        let user_username = user_username.into();
        let role_name = role_name.into();

        let pool = &self.db;

        // update user
        {
            let query = "UPDATE users
                               SET
                                user_roles = array_remove(user_roles, role_id),
                                user_modified_at = $1
                               FROM roles
                               WHERE
                                   user_username = $2
                                   AND role_name = $3
                               RETURNING role_id
                               ";

            let result = sqlx::query_as(query)
                .bind(XTimestamp(time as i64))
                .bind(&user_username)
                .bind(&role_name)
                .fetch_one(pool)
                .await;

            debug!("query {query}\nresults {result:?}");

            let (_role_id,): (i64,) = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbUserUpdateRoleRemoveErr::RoleNotFound { name: role_name });
                }
                Err(err) => {
                    error!("unexpected db error (post_update_file_add) {err}");
                    return Err(DbUserUpdateRoleRemoveErr::Db(err));
                }
            };
        }

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_update_role_remove() {
    init_log();

    let db = Db::test_db(0, "test_user_update_role_remove").await;

    let _user1 = {
        let invite = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
        db.user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
            .await
            .unwrap()
    };

    let _user2 = {
        let invite = db.invite_add(0, "hey2@heyadora.com", 1).await.unwrap();
        db.user_add(0, "hey2", "hey2", invite.token.clone(), 10, 10)
            .await
            .unwrap()
    };

    // assert success
    {
        let _role_one = db
            .role_add(0, "one", vec!["foo".to_string()])
            .await
            .unwrap();
        let _role_two = db
            .role_add(0, "two", vec!["bar".to_string()])
            .await
            .unwrap();
        let _role_three = db
            .role_add(0, "three", vec!["bar".to_string()])
            .await
            .unwrap();

        db.user_update_role_add(0, "hey", "one").await.unwrap();
        db.user_update_role_add(0, "hey", "two").await.unwrap();
        db.user_update_role_add(0, "hey", "three").await.unwrap();

        db.user_update_role_add(0, "hey2", "two").await.unwrap();

        let user = db.user_get_by_username("", "hey").await.unwrap();
        let user2 = db.user_get_by_username("", "hey2").await.unwrap();

        assert_eq!(user2.roles, vec![2]);
        assert_eq!(user.roles, vec![1, 2, 3]);

        db.user_update_role_remove(0, "hey", "two").await.unwrap();

        let user = db.user_get_by_username("", "hey").await.unwrap();
        let user2 = db.user_get_by_username("", "hey2").await.unwrap();

        assert_eq!(user.roles, vec![1, 3]);
        assert_eq!(user2.roles, vec![2]);
    }

    // assert error
    {
        let result = db.user_update_role_remove(0, "hey", "invalid").await;
        assert!(matches!(
            result,
            Err(DbUserUpdateRoleRemoveErr::RoleNotFound { .. })
        ));

        let user = db.user_get_by_username("", "hey").await.unwrap();
        let user2 = db.user_get_by_username("", "hey2").await.unwrap();

        assert_eq!(user.roles, vec![1, 3]);
        assert_eq!(user2.roles, vec![2]);
    }
}
