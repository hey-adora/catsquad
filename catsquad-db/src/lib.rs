use catsquad_log::prelude::*;
use surrealdb::engine::local::{Db as DbEngine, Mem};
use surrealdb::types::{RecordId, SurrealValue, ToSql};
use surrealdb::{IndexedResults, Surreal};

use crate::migration::migrate;

mod migration;
mod query;

pub use query::email_sent_add::DbEmailSent;
pub use query::email_sent_add::DbEmailSentAddErr;
pub use query::email_sent_add::DbEmailSentReason;
pub use query::email_sent_add::create_email_sent_id;
pub use query::invite_add::DbInvite;
pub use query::invite_add::DbInviteAddErr;
pub use query::invite_add::create_invite_id;
pub use query::invite_get_all::DbInviteGetAllErr;
pub use query::invite_get_by_key::DbInviteGetByKeyErr;
pub use query::migration_add::DbMigration;
pub use query::migration_add::DbMigrationAddErr;
pub use query::migration_get_latest::DbMigrationGetLatestErr;
pub use query::user_add::DbUser;
pub use query::user_add::DbUserAddErr;
pub use query::user_add::create_user_id;
pub use query::user_get_all::DbUserGetAllErr;
pub use query::user_get_by_email::DbUserGetByEmailErr;
pub use query::user_get_by_username::DbUserGetByUsernameErr;
pub use query::user_get_password::DbUserGetPasswordErr;
pub use query::user_update_password_by_email::DbUserUpdatePasswordByEmailErr;
pub use query::user_update_password_by_id::DbUserUpdatePasswordByIdErr;

pub fn id_to_string(v: RecordId) -> String {
    v.key.to_sql_pretty()
}

#[derive(Clone, Debug)]
pub struct Db {
    db: Surreal<DbEngine>,
}

impl Db {
    pub async fn mem() -> Self {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("catsquad").use_db("api").await.unwrap();
        let db = Self { db };
        migrate(&db).await;
        db
    }
}
trait SurrealCheckUtils {
    fn check_good<ERR: std::error::Error + From<surrealdb::Error>>(
        self,
        f: impl FnOnce(surrealdb::Error) -> ERR,
    ) -> Result<IndexedResults, ERR>;

    fn check_better<ERR: std::error::Error + From<surrealdb::Error>>(
        self,
        f: impl FnOnce(surrealdb::Error) -> ERR,
    ) -> Result<IndexedResults, ERR>;
}

trait SurrealSerializeUtils<ERR: std::error::Error + From<surrealdb::Error>> {
    fn and_then_take_all<Value: SurrealValue + serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
    ) -> Result<Vec<Value>, ERR>;
    fn and_then_take_or<Value: SurrealValue + serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
        err: ERR,
    ) -> Result<Value, ERR>;
    fn and_then_take_expect<Value: SurrealValue + serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
    ) -> Result<Value, ERR>;
}

impl SurrealCheckUtils for Result<IndexedResults, surrealdb::Error> {
    fn check_good<ERR: std::error::Error + From<surrealdb::Error>>(
        self,
        f: impl FnOnce(surrealdb::Error) -> ERR,
    ) -> Result<IndexedResults, ERR> {
        self.inspect_err(|err| error!("db error: {err}"))
            .inspect(|e| trace!("result {e:#?}"))?
            .check()
            .map_err(f)
    }

    fn check_better<ERR: std::error::Error + From<surrealdb::Error>>(
        self,
        f: impl FnOnce(surrealdb::Error) -> ERR,
    ) -> Result<IndexedResults, ERR> {
        let mut results = self.inspect_err(|err| error!("db error: {err}"))?;
        trace!("results {results:#?}");
        let errors = results.take_errors();

        let mut error_first = None;
        let mut error_thrown = None;
        let mut error_internal = None;
        for (i, error) in errors {
            match error {
                err if err.details().is_thrown() => {
                    error_thrown = Some(err);
                    break;
                }
                err if error.is_internal() => {
                    error_internal = Some(err);
                    break;
                }
                err => {
                    if error_first.is_none() {
                        error_first = Some(err);
                    }
                }
            }
        }

        let error = if error_thrown.is_some() {
            error_thrown
        } else if error_internal.is_some() {
            error_internal
        } else {
            error_first
        };

        trace!("error picked {error:?}");

        let results: Result<IndexedResults, surrealdb::Error> = match error {
            Some(err) => Err(err),
            None => Ok(results),
        };

        results.inspect(|e| trace!("result {e:#?}")).map_err(f)
    }
}

impl<ERR: std::error::Error + From<surrealdb::Error>> SurrealSerializeUtils<ERR>
    for Result<IndexedResults, ERR>
{
    fn and_then_take_all<Value: SurrealValue + serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
    ) -> Result<Vec<Value>, ERR> {
        self.and_then(|mut result| {
            result
                .take::<Vec<Value>>(index)
                .inspect(|v| trace!("db serialized to: {v:#?}"))
                .map_err(ERR::from)
        })
    }

    fn and_then_take_or<Value: serde::de::DeserializeOwned + std::fmt::Debug + SurrealValue>(
        self,
        index: usize,
        err: ERR,
    ) -> Result<Value, ERR> {
        self.and_then(|mut result| {
            result
                .take::<Option<Value>>(index)
                .inspect(|v| trace!("db serialized to: {v:#?}"))
                .map_err(ERR::from)
                .and_then(|v| v.ok_or(err))
        })
    }

    fn and_then_take_expect<Value: serde::de::DeserializeOwned + std::fmt::Debug + SurrealValue>(
        self,
        index: usize,
    ) -> Result<Value, ERR> {
        self.and_then(|mut result| {
            result
                .take::<Option<Value>>(index)
                .inspect(|v| trace!("db serialized to: {v:#?}"))
                .map_err(ERR::from)
                .map(|v| v.expect("must exist"))
        })
    }
}

trait SurrealErrUtils {
    fn table_not_found(&self, table_name: impl AsRef<str>) -> bool;
    fn index_exists(&self, index_name: impl AsRef<str>) -> bool;
    fn field_value_null(&self, field_name: impl AsRef<str>) -> bool;
    fn thrown(&self, field_name: impl AsRef<str>) -> bool;
}

impl SurrealErrUtils for surrealdb::Error {
    fn table_not_found(&self, table_name: impl AsRef<str>) -> bool {
        //The table 'migration' does not exist
        let msg = self.message();
        let mut needle = String::from("The table '");
        needle.push_str(table_name.as_ref());
        needle.push_str("' does not exist");
        msg == needle
    }

    fn index_exists(&self, index_name: impl AsRef<str>) -> bool {
        // "Database index `idx_user_email` already contains 'hey@hey.com', with record `user:tjateqrc93xqjfctf561`"
        let msg = self.message();
        // TODO optimize string allocation size thing
        let mut needle = String::from("Database index `");
        needle.push_str(index_name.as_ref());
        needle.push_str("` already contains");
        let to = needle.len();
        if to > msg.len() {
            return false;
        }
        &msg[0..to] == needle
    }

    fn field_value_null(&self, field_name: impl AsRef<str>) -> bool {
        let msg = self.message();
        // TODO optimize string allocation size thing
        let mut needle = String::from("Couldn't coerce value for field `");
        needle.push_str(field_name.as_ref());
        needle.push('`');
        let to = needle.len();
        if to > msg.len() {
            return false;
        }
        &msg[0..to] == needle
    }

    fn thrown(&self, throw_msg: impl AsRef<str>) -> bool {
        let msg = self.message();
        let mut needle = String::from("An error occurred: ");
        needle.push_str(throw_msg.as_ref());
        msg == needle
    }
}
