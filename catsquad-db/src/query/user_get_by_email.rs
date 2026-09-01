use crate::{Db, DbUser};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetByEmailErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

impl Db {
    pub async fn user_get_by_email(
        &self,
        email: impl Into<String>,
    ) -> Result<DbUser, DbUserGetByEmailErr> {
        let pool = &self.db;
        let email = email.into();

        let query = "SELECT * FROM users WHERE user_email = $1";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).bind(email).fetch_one(pool).await;

        let users = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbUserGetByEmailErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserGetByEmailErr::Db(err));
            }
        };

        Ok(users)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_get_by_email() {
    init_log();

    let db = Db::test_db(0, "test_user_get_by_email").await;

    // add user 1
    {
        let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
        db.user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
            .await
            .unwrap();
    }

    // add user 2
    {
        let invite = db.invite_add(0, "hey2@hey.com", 10).await.unwrap();
        db.user_add(0, "hey2", "hey2", invite.token.clone(), 10, 10)
            .await
            .unwrap();
    }

    let user = db.user_get_by_email("hey@hey.com").await.unwrap();
    assert_eq!(user.email, "hey@hey.com");
}
