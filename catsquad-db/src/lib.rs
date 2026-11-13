use std::fmt::Display;

pub use query::invite_add::*;
use sqlx::{
    AssertSqlSafe, Connection, Decode, Encode, PgConnection, Pool, Postgres,
    postgres::{PgArgumentBuffer, PgPoolOptions, PgTypeInfo, PgValueRef},
};

mod migration;
mod query;

pub use query::comment_add::*;
pub use query::comment_get_all::*;
pub use query::comment_remove::*;
pub use query::comment_search::*;
pub use query::comment_update_text::*;
pub use query::confirm_token_add::*;
pub use query::confirm_token_get_all::*;
pub use query::confirm_token_update_confirm_tx::*;
pub use query::email_change_add::*;
pub use query::email_change_get_all::*;
pub use query::email_change_get_by_id::*;
pub use query::email_change_update_cancel::*;
pub use query::email_change_update_current_confirm::*;
pub use query::email_change_update_finish::*;
pub use query::email_change_update_new_add::*;
pub use query::email_change_update_new_confirm::*;
pub use query::email_sent_add::*;
pub use query::file_image_add::*;
pub use query::file_image_add_tx::*;
pub use query::file_image_get_all::*;
pub use query::file_image_get_by_hash::*;
pub use query::file_image_remove_tx::*;
pub use query::follow_add::*;
pub use query::follow_get_all::*;
pub use query::follow_remove::*;
pub use query::invite_add::*;
pub use query::invite_get_all::*;
pub use query::invite_get_by_id::*;
pub use query::migration_add::*;
pub use query::migration_get_latest::*;
pub use query::post_add::*;
pub use query::post_get_all::*;
pub use query::post_get_by_id::*;
pub use query::post_image_get_by_hash::*;
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
pub use query::post_update_state::*;
pub use query::post_update_tags::*;
pub use query::post_update_title::*;
pub use query::role_add::*;
pub use query::role_get_all::*;
pub use query::role_remove::*;
pub use query::session_add::*;
pub use query::session_get_all::*;
pub use query::session_get_by_id::*;
pub use query::session_remove::*;
pub use query::user_add::*;
pub use query::user_get_all::*;
pub use query::user_get_by_email::*;
pub use query::user_get_by_session::*;
pub use query::user_get_by_username::*;
pub use query::user_get_password::*;
pub use query::user_remove_by_token::*;
pub use query::user_remove_by_usernames::*;
pub use query::user_search_builder::*;
pub use query::user_search_users::*;
pub use query::user_search_users::*;
pub use query::user_update_aboutme::*;
pub use query::user_update_password_by_email::*;
pub use query::user_update_password_by_token::*;
pub use query::user_update_password_by_username::*;
pub use query::user_update_pfp::*;
pub use query::user_update_role_add::*;
pub use query::user_update_role_remove::*;
pub use query::user_update_state::*;
pub use query::user_update_support::*;
pub use query::user_update_username::*;

const POSTGRES_TIME_EPOCH_OFFSET: i64 = 946684800000000;

pub type Uuid = [u8; 16];

#[derive(Clone, Debug)]
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
        let micros = self.0 - POSTGRES_TIME_EPOCH_OFFSET;
        Encode::<Postgres>::encode(micros, buf)
    }
}

impl<'r> Decode<'r, Postgres> for XTimestamp {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let mut micros: i64 = Decode::<Postgres>::decode(value)?;
        micros += POSTGRES_TIME_EPOCH_OFFSET;
        Ok(Self(micros))
    }
}

impl sqlx::Type<Postgres> for XTimestamp {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("timestamp")
    }
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
    TInputItem: Clone,
    TCallbackOutput: AsRef<str>,
    TInput: IntoIterator<Item = TInputItem>,
{
    let mut output = String::new();
    let mut iter = input.into_iter();

    for item in &mut iter {
        let result = callback(item);
        let result = result.as_ref();
        if result.is_empty() {
            continue;
        }
        output.push_str(&result);
        break;
    }

    for item in iter {
        let result = callback(item);
        let result = result.as_ref();
        if result.is_empty() {
            continue;
        }
        output.push_str(between);
        output.push_str(&result);
    }

    output
}

pub fn run_if_not_empty(input: impl AsRef<str>, callback: impl FnOnce() -> String) -> String {
    let input = input.as_ref();

    let result = if !input.is_empty() {
        callback()
    } else {
        "".to_string()
    };

    result
}

#[test]
pub fn test_join_str() {
    let filters = ["a", "", "b", ""];
    let result = join_str(filters, " AND ", |v| v);
    assert_eq!(result, "a AND b");

    let filters = ["", "a", "b", ""];
    let result = join_str(filters, " AND ", |v| v);
    assert_eq!(result, "a AND b");
}
