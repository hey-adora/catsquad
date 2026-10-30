use crate::{Db, XTimestamp, join_str};
use catsquad_log::prelude::*;
use catsquad_shared::{Order, TimeRange};
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbUserSearchBuilderErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbUserSearch {
    #[sqlx(rename = "user_username")]
    pub username: String,
    #[sqlx(rename = "user_email")]
    pub email: String,
    #[sqlx(rename = "user_support")]
    pub support: String,
    #[sqlx(rename = "user_aboutme")]
    pub aboutme: String,
    #[sqlx(rename = "user_pfp_image_hash")]
    pub pfp_image_hash: i64,
    #[sqlx(default, rename = "user_pfp_image_proccesed")]
    pub pfp_image_proccesed: bool,
    #[sqlx(rename = "user_likes_count")]
    #[sqlx(try_from = "i64")]
    pub likes_count: u32,
    #[sqlx(rename = "user_followers_count")]
    #[sqlx(try_from = "i64")]
    pub followers_count: u32,
    #[sqlx(rename = "user_following_count")]
    #[sqlx(try_from = "i64")]
    pub following_count: u32,
    #[sqlx(rename = "user_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

pub enum DbUserSearchBuilder {
    Users(Vec<DbUserSearch>),
    PageCount(u32),
}

impl Db {
    pub async fn user_search_builder(
        &self,
        pagination: bool,
        time_range_value: u64,
        limit: usize,
        offset: usize,
        time_range: TimeRange,
        order: Order,
    ) -> Result<DbUserSearchBuilder, DbUserSearchBuilderErr> {
        let pool = &self.db;

        let q_order = match order {
            Order::OneTwoThree => "ASC",
            Order::ThreeTwoOne => "DESC",
        };

        let q_order = if pagination {
            "".to_string()
        } else {
            format!("ORDER BY user_created_at {q_order}")
        };

        let q_time_after = match time_range {
            TimeRange::None => "",
            TimeRange::Less => "user_created_at < $1",
            TimeRange::LessOrEqual => "user_created_at <= $1",
            TimeRange::More => "user_created_at > $1",
            TimeRange::MoreOrEqual => "user_created_at >= $1",
        }
        .to_string();

        let filters = [q_time_after];
        let q_where = join_str(filters, " AND ", |v| v);
        let q_where = if !q_where.is_empty() {
            format!("WHERE {q_where}")
        } else {
            String::new()
        };

        let q_limit = if limit == 0 { "" } else { "LIMIT $2" };

        let q_offset = if offset == 0 { "" } else { "OFFSET $3" };

        let q_selection = if pagination {
            "count(1)"
        } else {
            "
             user_username,
             user_email,
             user_support,
             user_aboutme,
             user_pfp_image_hash,
             coalesce(image_processed, false) as user_pfp_image_proccesed,
             user_likes_count,
             user_followers_count,
             user_following_count,
             user_created_at
            "
        };

        let query_str = format!(
            "
             SELECT
                 {q_selection}
             FROM users 
             LEFT JOIN files_images
                ON user_pfp_image_hash = image_hash
             {q_where}
             {q_order}
             {q_limit}
             {q_offset}
            "
        );

        let query = AssertSqlSafe(query_str.clone());

        if pagination {
            let result = sqlx::query_as(query)
                .bind(XTimestamp(time_range_value as i64))
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_one(pool)
                .await;

            debug!("query {query_str}\nresult {result:#?}");

            let (page_count,): (i64,) = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserSearchBuilderErr::Db(err));
                }
            };

            return Ok(DbUserSearchBuilder::PageCount(page_count as u32));
        }

        let result = sqlx::query_as(query)
            .bind(XTimestamp(time_range_value as i64))
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await;

        debug!("query {query_str}\nresult {result:#?}");

        let users: Vec<DbUserSearch> = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserSearchBuilderErr::Db(err));
            }
        };

        Ok(DbUserSearchBuilder::Users(users))
    }
}
