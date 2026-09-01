use crate::{Db, DbUser, Uuid, XTimestamp, XUuid};
use catsquad_log::prelude::*;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbPasswordChange {
    #[sqlx(rename = "password_change_token")]
    #[sqlx(try_from = "XUuid")]
    pub token: Uuid,
    #[sqlx(rename = "password_change_user_email")]
    pub user_email: String,
    #[sqlx(rename = "password_change_used")]
    pub used: bool,
    #[sqlx(rename = "password_change_expires_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub expires_at: u64,
    #[sqlx(rename = "password_change_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "password_change_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbPasswordChangeAddErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user with \"{0}\" email not found")]
    UserNotFound(String),
}

impl Db {
    pub async fn password_change_define(&self) {
        let pool = &self.db;
        // add foreign key FILE
        let query = "
            CREATE TABLE passwords_changes (
                password_change_token uuid PRIMARY KEY DEFAULT uuidv7(),
                password_change_user_email varchar NOT NULL references users(user_email) ON UPDATE CASCADE ON DELETE CASCADE,
                password_change_used bool DEFAULT FALSE,
                password_change_expires_at timestamp NOT NULL,
                password_change_modified_at timestamp NOT NULL,
                password_change_created_at timestamp NOT NULL
            );
        ";
        // CREATE INDEX post_user_username_idx ON posts (post_user_username);
        // CREATE INDEX post_username_and_state_idx ON posts (post_user_username, post_state);
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
        // let query = "
        //         DEFINE TABLE password_change SCHEMAFULL;
        //         DEFINE FIELD user ON TABLE password_change TYPE record<user>;
        //         DEFINE FIELD expires ON TABLE password_change TYPE number;
        //         DEFINE FIELD used ON TABLE password_change TYPE bool;
        //         DEFINE FIELD modified_at ON TABLE password_change TYPE number;
        //         DEFINE FIELD created_at ON TABLE password_change TYPE number;
        //     ";
        // trace!("about to run {query}");
        // self.db.query(query).await.unwrap().check().unwrap();
    }

    pub async fn password_change_add(
        &self,
        time: u64,
        email: impl Into<String>,
        expires: u64,
    ) -> Result<DbPasswordChange, DbPasswordChangeAddErr> {
        let pool = &self.db;
        let email = email.into();

        let query = "
            INSERT INTO passwords_changes (
                    password_change_user_email,
                    password_change_expires_at,
                    password_change_modified_at,
                    password_change_created_at
                )
                VALUES ( $1, $2, $3, $3 )
                RETURNING password_change_token
        ";

        debug!("about to run {query}");

        let result = sqlx::query_as(query)
            .bind(&email)
            .bind(XTimestamp(expires as i64))
            .bind(XTimestamp(time as i64))
            .fetch_one(pool)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::Database(err))
                if err.is_foreign_key_violation()
                    && err.constraint()
                        == Some("passwords_changes_password_change_user_email_fkey") =>
            {
                return Err(DbPasswordChangeAddErr::UserNotFound(email));
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPasswordChangeAddErr::Db(err));
            }
        };

        let (XUuid(token),): (XUuid,) = result;

        let password_change = DbPasswordChange {
            token,
            user_email: email,
            used: false,
            expires_at: expires,
            modified_at: time,
            created_at: time,
        };

        Ok(password_change)
        // let email: String = email.into();

        // let query = r#"
        //          BEGIN TRANSACTION;
        //          LET $user = SELECT id FROM ONLY user WHERE email = $email;
        //          CREATE password_change SET
        //                user = $user.id,
        //                expires = $expires,
        //                used = false,
        //                modified_at = $time,
        //                created_at = $time
        //                RETURN *, user.*;
        //         COMMIT TRANSACTION;
        //         "#;
        // trace!("about to run {query}");

        // self.db
        //     .query(query)
        //     .bind(("time", time))
        //     .bind(("email", email.clone()))
        //     .bind(("expires", expires))
        //     .await
        //     .check_better(|err| match err {
        //         err if err.field_value_null("user") => DbPasswordChangeAddErr::UserNotFound(email),
        //         err => {
        //             error!("unexpected db error {err}");
        //             DbPasswordChangeAddErr::Db(err)
        //         }
        //     })
        //     .and_then_take_expect(2)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_password_change_add() {
    init_log();

    let db = Db::test_db(0, "test_password_change_add").await;

    let email = "hey@heyadora.com";
    let invite1 = db.invite_add(0, email, 1).await.unwrap();
    assert_eq!(invite1.email, email);

    let _user = db
        .user_add(0, "hey", "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let _result = db.password_change_add(0, email, 10).await.unwrap();

    let result = db.password_change_add(0, "invalid", 10).await;
    // matches!()
    assert!(matches!(
        result,
        Err(DbPasswordChangeAddErr::UserNotFound(_))
    ));
}
