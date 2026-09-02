use std::fmt::Display;

pub use query::invite_add::*;
use sqlx::{
    AssertSqlSafe, Connection, Decode, Encode, PgConnection, Pool, Postgres,
    postgres::{PgArgumentBuffer, PgPoolOptions, PgTypeInfo, PgValueRef},
};

// use std::mem;

// use catsquad_log::prelude::*;
// use chrono::{DateTime, Utc};
// use sqlx::encode::IsNull;
// use sqlx::postgres::{PgArgumentBuffer, PgPoolOptions, PgTypeInfo};
// use sqlx::{AssertSqlSafe, Encode, Execute, Pool, Postgres};
// use surrealdb::engine::local::{Db as DbEngine, Mem, SurrealKv};
// use surrealdb::engine::remote::ws::{Client, Ws};
// use surrealdb::opt::auth::Root;
// use surrealdb::types::{RecordId, SurrealValue, ToSql};
// use surrealdb::{Connection, IndexedResults, Surreal};

// use crate::migration::migrate;

mod migration;
mod query;

pub use query::comment_add::*;
// pub use query::comment_get_all::*;
// pub use query::comment_remove::*;
// pub use query::comment_search::*;
// pub use query::comment_update_text::*;
pub use query::email_change_add::*;
pub use query::email_change_get_by_key::*;
pub use query::email_change_update_cancel::*;
pub use query::email_change_update_current_confirm::*;
pub use query::email_change_update_finish::*;
pub use query::email_change_update_new_add::*;
pub use query::email_change_update_new_confirm::*;
pub use query::email_sent_add::*;
// pub use query::email_sent_get_all;
pub use query::file_image_add::*;
pub use query::file_image_get_by_hash::*;
pub use query::invite_add::*;
pub use query::invite_get_all::*;
pub use query::invite_get_by_key::*;
pub use query::migration_add::*;
pub use query::migration_get_latest::*;
pub use query::password_change_add::*;
pub use query::password_change_get_all::*;
pub use query::password_change_update_confirm::*;
pub use query::post_add::*;
pub use query::post_get_all::*;
pub use query::post_get_by_key::*;
// pub use query::post_get_unproccesed::*;
pub use query::post_like_add::*;
pub use query::post_like_exists_by_post::*;
pub use query::post_like_get_all::*;
pub use query::post_like_remove::*;
pub use query::post_remove::*;
pub use query::post_search::*;
pub use query::post_update_builder_text::*;
pub use query::post_update_description::*;
pub use query::post_update_file_add::*;
pub use query::post_update_file_remove::*;
pub use query::post_update_order::*;
// pub use query::post_update_proccesed::*;
pub use query::post_update_state::*;
pub use query::post_update_tags::*;
pub use query::post_update_title::*;
pub use query::session_add::*;
pub use query::session_get_by_key::*;
pub use query::session_remove::*;
pub use query::user_add::*;
pub use query::user_get_all::*;
pub use query::user_get_by_email::*;
pub use query::user_get_by_username::*;
pub use query::user_get_password::*;
pub use query::user_update_password_by_email::*;
pub use query::user_update_password_by_username::*;
pub use query::user_update_username::*;

// pub fn id_to_string(v: RecordId) -> String {
//     v.key.to_sql_pretty()
// }

// pub type DbKindLocal = DbEngine;

pub type Uuid = [u8; 16];

#[derive(Clone)]
pub struct Db {
    db: Pool<Postgres>,
}

#[derive(Clone, Copy, Debug)]
pub struct XUuid(pub [u8; 16]);

impl From<Uuid> for XUuid {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<XUuid> for Uuid {
    fn from(value: XUuid) -> Self {
        value.0
    }
}

impl sqlx::Type<Postgres> for XUuid {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("uuid")
    }
}

impl Encode<'_, Postgres> for XUuid {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let bytes = self.0;
        Encode::<Postgres>::encode(bytes, buf)
    }
}

impl<'r> Decode<'r, Postgres> for XUuid {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let bytes = Decode::<Postgres>::decode(value)?;
        Ok(Self(bytes))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct XTimestamp(pub i64);

impl From<XTimestamp> for u64 {
    fn from(value: XTimestamp) -> Self {
        value.0 as u64
    }
}

impl From<u64> for XTimestamp {
    fn from(value: u64) -> Self {
        Self(value as i64)
    }
}

// impl TryFrom<u64> for TimeStamp {
//     type Error = ;
//     fn try_from(value: u64) -> Result<Self, Self::Error> {
//         Ok(Self(value as i64))
//     }
// }

impl Encode<'_, Postgres> for XTimestamp {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let micros = self.0 - 946684800000000;
        Encode::<Postgres>::encode(micros, buf)
    }
}

impl<'r> Decode<'r, Postgres> for XTimestamp {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let mut micros: i64 = Decode::<Postgres>::decode(value)?;
        micros += 946684800000000_i64;
        Ok(Self(micros))
    }
}

impl sqlx::Type<Postgres> for XTimestamp {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("timestamp")
    }
}

// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub struct WrapU128(pub u128);

// // fn postgres_epoch_datetime() -> NaiveDateTime {
// //     NaiveDate::fkrom_ymd_opt(2000, 1, 1)
// //         .expect("expected 2000-01-01 to be a valid NaiveDate")
// //         .and_hms_opt(0, 0, 0)
// //         .expect("expected 2000-01-01T00:00:00 to be a valid NaiveDateTime")
// // }

// impl Encode<'_, Postgres> for WrapU128 {
//     fn encode_by_ref(
//         &self,
//         buf: &mut PgArgumentBuffer,
//     ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
//         let bytes = self.0.to_be_bytes();
//         // let postgres_epoch = 946684800000;
//         // let micros = (self.0 / 1000) as i64;
//         // let micros = postgres_epoch + micros;
//         // postgres_epoch_datetime();
//         // let nanos = self.0 as i64;
//         // Encode::<Postgres>::encode(micros, buf)
//         buf.extend(bytes);
//         Ok(IsNull::No)
//     }

//     // fn size_hint(&self) -> usize {
//     //     mem::size_of::<i64>()
//     // }
// }

// impl sqlx::Type<Postgres> for WrapU128 {
//     fn type_info() -> PgTypeInfo {
//         PgTypeInfo::with_name("uint16")
//     }
// }

// // #[derive(sqlx::Type)]
// // #[sqlx(transparent, type_name = "uint4", pg_format = "text")]
// // #[derive(Encode)]
// // #[sqlx(transparent, type_name = "uint4")]
// pub struct WrapU32(pub u32);

// impl Encode<'_, Postgres> for WrapU32 {
//     fn encode_by_ref(
//         &self,
//         buf: &mut PgArgumentBuffer,
//     ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
//         // let text = self.0.to_string();
//         let bytes = self.0.to_be_bytes();
//         buf.extend(bytes);
//         Ok(IsNull::No)
//     }
// }

// impl sqlx::Type<Postgres> for WrapU32 {
//     fn type_info() -> PgTypeInfo {
//         PgTypeInfo::with_name("uint4")
//     }
// }

// pub struct WrapU64(pub u32);

//
impl Db {
    // pub async fn user_define(&self) {
    //     // use query::user_add::DbUser;
    //     let pool = &self.db;

    //     let result = sqlx::raw_sql(
    //         "
    //         CREATE TABLE users (
    //             user_id int8 PRIMARY KEY generated always as identity,
    //             user_used_storage_bytes int8 NOT NULL DEFAULT 0,
    //             user_max_storage_per_file_bytes int8 NOT NULL,
    //             user_max_storage_bytes int8 NOT NULL,
    //             user_username varchar(32) NOT NULL,
    //             user_email varchar(100) NOT NULL,
    //             user_password varchar(255) NOT NULL,
    //             user_modified_at timestamp NOT NULL,
    //             user_created_at timestamp NOT NULL
    //         );
    //     ",
    //     )
    //     .execute(pool)
    //     .await
    //     .unwrap();

    //     debug!("{result:?}");
    // }

    // pub async fn user_add(
    //     &self,
    //     time: DateTime<Utc>,
    //     username: impl Into<String>,
    //     password: impl Into<String>,
    //     invite_token: i64,
    //     max_storage: u32,
    //     max_storage_per_file: u32,
    // ) {
    //     // use query::user_add::DbUser;
    //     // use std::time::Duration;

    //     let pool = &self.db;

    //     // let time = Duration::from_nanos_u128(time);
    //     // let time = DateTime::from_timestamp_nanos(time as i64);
    //     // let time = time + chrono::Duration::seconds(1);
    //     // let time = WrapU128(time);
    //     let username = username.into();
    //     let password = password.into();
    //     let invite_token = invite_token;
    //     let max_storage = max_storage as i64;
    //     let max_storage_per_file = max_storage_per_file as i64;

    //     // sqlx::Encode

    //     let result = sqlx::query(
    //         "
    //         INSERT INTO users (
    //                 user_max_storage_bytes,
    //                 user_max_storage_per_file_bytes,
    //                 user_username,
    //                 user_email,
    //                 user_password,
    //                 user_modified_at,
    //                 user_created_at
    //             )
    //             VALUES ( $1, $2, $3, $4, $5, $6, $7 )
    //             RETURNING user_id;
    //     ",
    //     )
    //     .bind(max_storage)
    //     .bind(max_storage_per_file)
    //     .bind(username)
    //     .bind(invite_token)
    //     .bind(password)
    //     .bind(time)
    //     .bind(time);
    //     // result.sql()
    //     // let sql = result.sql();
    //     // debug!("running query\n{result:#?}");
    //     let result = result.fetch_one(pool).await.unwrap();

    //     debug!("{result:?}");
    // }
}

async fn create_con(db_name: impl Display) -> PgConnection {
    let db_url = format!("postgres://hey@%2Fhome%2Fhey%2FProjects%2Fcatsquad%2Ftarget/{db_name}");
    PgConnection::connect(&db_url).await.unwrap()
}

async fn create_pool(db_name: impl Display) -> Result<Pool<Postgres>, sqlx::Error> {
    let db_url = format!("postgres://hey@%2Fhome%2Fhey%2FProjects%2Fcatsquad%2Ftarget/{db_name}");
    PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
}

impl Db {
    pub async fn test_db(time: u64, name: impl AsRef<str>) -> Self {
        let name = name.as_ref();
        Self::drop_database(name).await;
        Self::create(time, name).await
    }

    // pub async

    pub async fn drop_database(name: impl AsRef<str>) {
        let name = name.as_ref();
        let pool = create_pool("postgres").await.unwrap();
        let query = AssertSqlSafe(format!("DROP DATABASE {name};"));
        let _result = sqlx::raw_sql(query).execute(&pool).await;
    }

    pub async fn create(time: u64, name: impl AsRef<str>) -> Self {
        let name = name.as_ref();
        let result = create_pool(name).await;

        let pool = match result {
            Ok(v) => v,
            Err(err)
                if err
                    .to_string()
                    .contains(&format!("database \"{name}\" does not exist")) =>
            {
                {
                    let pool = create_pool("postgres").await.unwrap();
                    let query = AssertSqlSafe(format!("CREATE DATABASE {name};"));
                    let _result = sqlx::raw_sql(query).execute(&pool).await.unwrap();
                }

                let pool = create_pool(name).await.unwrap();

                // let _result = sqlx::raw_sql(
                //     "
                //         CREATE EXTENSION system_stats;
                //         CREATE EXTENSION vector;
                //         CREATE EXTENSION vchord CASCADE;
                //         CREATE EXTENSION uint128;
                //     ",
                // )
                let _result = sqlx::raw_sql(
                    "
                        CREATE EXTENSION system_stats;
                    ",
                )
                .execute(&pool)
                .await
                .unwrap();

                pool
            }
            Err(err) => {
                panic!("{}", err);
            }
        };

        let db = Self { db: pool };
        db.migrate(time).await;
        db
    }
}

pub fn join_str<TInput, TInputItem, TCallbackOutput>(
    input: TInput,
    between: &str,
    mut callback: impl FnMut(TInputItem) -> TCallbackOutput,
) -> String
where
    TCallbackOutput: AsRef<str>,
    TInput: IntoIterator<Item = TInputItem>,
{
    let mut output = String::new();

    let mut iter = input.into_iter().peekable();

    loop {
        let Some(item) = iter.next() else {
            break;
        };
        let result = callback(item);
        let result = result.as_ref();

        if result.is_empty() {
            continue;
        }

        output.push_str(&result);

        let next_is_empty = iter.peek();

        if next_is_empty.is_none() {
            break;
        }

        output.push_str(between);
    }

    output
}

pub fn if_empty(input: impl AsRef<str>, callback: impl FnOnce() -> String) -> String {
    let input = input.as_ref();

    let result = if !input.is_empty() {
        callback()
    } else {
        "".to_string()
    };

    result
}

// #[derive(Clone, Debug)]
// pub struct Db<C: Connection> {
//     db: Surreal<C>,
// }

// impl Db<DbEngine> {
//     pub async fn mem(time: u128) -> Self {
//         let db = Surreal::new::<Mem>(()).await.unwrap();
//         db.use_ns("catsquad").use_db("api").await.unwrap();
//         let db = Self { db };
//         migrate(time, &db).await;
//         db
//     }
//     pub async fn local(time: u128, database_path: impl AsRef<std::path::Path>) -> Self {
//         let db = Surreal::new::<SurrealKv>(database_path.as_ref())
//             .await
//             .unwrap();
//         db.use_ns("catsquad").use_db("api").await.unwrap();
//         let db = Self { db };
//         migrate(time, &db).await;
//         db
//     }
// }
// impl Db<Client> {
//     pub async fn ws(
//         time: u128,
//         namespace: impl AsRef<str>,
//         db_name: impl AsRef<str>,
//         connection_url: impl AsRef<str>,
//     ) -> Self {
//         let connection_url = connection_url.as_ref();
//         info!("connection to {connection_url}");
//         let db = Surreal::new::<Ws>(connection_url).await.unwrap();
//         // db.signin(Root {
//         //     username: "root".to_string(),
//         //     password: "root".to_string(),
//         // })
//         // .await
//         // .unwrap();
//         db.use_ns(namespace.as_ref())
//             .use_db(db_name.as_ref())
//             .await
//             .unwrap();
//         let db = Self { db };
//         migrate(time, &db).await;
//         db
//     }
// }
// // impl<C: Connection> Db<C> {
// //     pub async fn mem(time: u128) -> Self {
// //         let db = Surreal::new::<Mem>(()).await.unwrap();
// //         db.use_ns("catsquad").use_db("api").await.unwrap();
// //         let db = Self { db };
// //         migrate(time, &db).await;
// //         db
// //     }
// //     pub async fn local(time: u128, database_path: impl AsRef<std::path::Path>) -> Self {
// //         let db = Surreal::new::<SurrealKv>(database_path.as_ref())
// //             .await
// //             .unwrap();
// //         db.use_ns("catsquad").use_db("api").await.unwrap();
// //         let db = Self { db };
// //         migrate(time, &db).await;
// //         db
// //     }
// //     pub async fn ws(time: u128, connection_url: impl AsRef<str>) -> Self {
// //         let db = Surreal::new::<Ws>(connection_url.as_ref()).await.unwrap();
// //         db.use_ns("catsquad").use_db("api").await.unwrap();
// //         let db = Self { db };
// //         migrate(time, &db).await;
// //         db
// //     }
// // }
// trait SurrealCheckUtils {
//     fn check_good<ERR: std::error::Error + From<surrealdb::Error>>(
//         self,
//         f: impl FnOnce(surrealdb::Error) -> ERR,
//     ) -> Result<IndexedResults, ERR>;

//     fn check_better<ERR: std::error::Error + From<surrealdb::Error>>(
//         self,
//         f: impl FnOnce(surrealdb::Error) -> ERR,
//     ) -> Result<IndexedResults, ERR>;
// }

// trait SurrealSerializeUtils<ERR: std::error::Error + From<surrealdb::Error>> {
//     fn and_then_take_all<Value: SurrealValue + serde::de::DeserializeOwned + std::fmt::Debug>(
//         self,
//         index: usize,
//     ) -> Result<Vec<Value>, ERR>;
//     fn and_then_take_or<Value: SurrealValue + serde::de::DeserializeOwned + std::fmt::Debug>(
//         self,
//         index: usize,
//         err: ERR,
//     ) -> Result<Value, ERR>;
//     fn and_then_take_expect<Value: SurrealValue + serde::de::DeserializeOwned + std::fmt::Debug>(
//         self,
//         index: usize,
//     ) -> Result<Value, ERR>;
// }

// impl SurrealCheckUtils for Result<IndexedResults, surrealdb::Error> {
//     fn check_good<ERR: std::error::Error + From<surrealdb::Error>>(
//         self,
//         f: impl FnOnce(surrealdb::Error) -> ERR,
//     ) -> Result<IndexedResults, ERR> {
//         self.inspect_err(|err| error!("db error: {err}"))
//             .inspect(|e| trace!("result {e:#?}"))?
//             .check()
//             .map_err(f)
//     }

//     fn check_better<ERR: std::error::Error + From<surrealdb::Error>>(
//         self,
//         f: impl FnOnce(surrealdb::Error) -> ERR,
//     ) -> Result<IndexedResults, ERR> {
//         let mut results = self.inspect_err(|err| error!("db error: {err}"))?;
//         trace!("results {results:#?}");
//         let errors = results.take_errors();

//         let mut error_first = None;
//         let mut error_thrown = None;
//         let mut error_internal = None;
//         for (i, error) in errors {
//             match error {
//                 err if err.details().is_thrown() => {
//                     error_thrown = Some(err);
//                     break;
//                 }
//                 err if error.is_internal() => {
//                     error_internal = Some(err);
//                     break;
//                 }
//                 err => {
//                     if error_first.is_none() {
//                         error_first = Some(err);
//                     }
//                 }
//             }
//         }

//         let error = if error_thrown.is_some() {
//             error_thrown
//         } else if error_internal.is_some() {
//             error_internal
//         } else {
//             error_first
//         };

//         trace!("error picked {error:?}");

//         let results: Result<IndexedResults, surrealdb::Error> = match error {
//             Some(err) => Err(err),
//             None => Ok(results),
//         };

//         results.inspect(|e| trace!("result {e:#?}")).map_err(f)
//     }
// }

// impl<ERR: std::error::Error + From<surrealdb::Error>> SurrealSerializeUtils<ERR>
//     for Result<IndexedResults, ERR>
// {
//     fn and_then_take_all<Value: SurrealValue + serde::de::DeserializeOwned + std::fmt::Debug>(
//         self,
//         index: usize,
//     ) -> Result<Vec<Value>, ERR> {
//         self.and_then(|mut result| {
//             result
//                 .take::<Vec<Value>>(index)
//                 .inspect(|v| trace!("db serialized to: {v:#?}"))
//                 .map_err(ERR::from)
//         })
//     }

//     fn and_then_take_or<Value: serde::de::DeserializeOwned + std::fmt::Debug + SurrealValue>(
//         self,
//         index: usize,
//         err: ERR,
//     ) -> Result<Value, ERR> {
//         self.and_then(|mut result| {
//             result
//                 .take::<Option<Value>>(index)
//                 .inspect(|v| trace!("db serialized to: {v:#?}"))
//                 .map_err(ERR::from)
//                 .and_then(|v| v.ok_or(err))
//         })
//     }

//     fn and_then_take_expect<Value: serde::de::DeserializeOwned + std::fmt::Debug + SurrealValue>(
//         self,
//         index: usize,
//     ) -> Result<Value, ERR> {
//         self.and_then(|mut result| {
//             result
//                 .take::<Option<Value>>(index)
//                 .inspect(|v| trace!("db serialized to: {v:#?}"))
//                 .map_err(ERR::from)
//                 .map(|v| v.expect("must exist"))
//         })
//     }
// }

// trait SurrealErrUtils {
//     fn table_not_found(&self, table_name: impl AsRef<str>) -> bool;
//     fn index_exists(&self, index_name: impl AsRef<str>) -> bool;
//     fn field_value_null(&self, field_name: impl AsRef<str>) -> bool;
//     fn thrown(&self, field_name: impl AsRef<str>) -> bool;
// }

// impl SurrealErrUtils for surrealdb::Error {
//     fn table_not_found(&self, table_name: impl AsRef<str>) -> bool {
//         //The table 'migration' does not exist
//         let msg = self.message();
//         let mut needle = String::from("The table '");
//         needle.push_str(table_name.as_ref());
//         needle.push_str("' does not exist");
//         msg == needle
//     }

//     fn index_exists(&self, index_name: impl AsRef<str>) -> bool {
//         // "Database index `idx_user_email` already contains 'hey@hey.com', with record `user:tjateqrc93xqjfctf561`"
//         let msg = self.message();
//         // TODO optimize string allocation size thing
//         let mut needle = String::from("Database index `");
//         needle.push_str(index_name.as_ref());
//         needle.push_str("` already contains");
//         let to = needle.len();
//         if to > msg.len() {
//             return false;
//         }
//         &msg[0..to] == needle
//     }

//     fn field_value_null(&self, field_name: impl AsRef<str>) -> bool {
//         let msg = self.message();
//         // TODO optimize string allocation size thing
//         let mut needle = String::from("Couldn't coerce value for field `");
//         needle.push_str(field_name.as_ref());
//         needle.push('`');
//         let to = needle.len();
//         if to > msg.len() {
//             return false;
//         }
//         &msg[0..to] == needle
//     }

//     fn thrown(&self, throw_msg: impl AsRef<str>) -> bool {
//         let msg = self.message();
//         let mut needle = String::from("An error occurred: ");
//         needle.push_str(throw_msg.as_ref());
//         msg == needle
//     }
// }
