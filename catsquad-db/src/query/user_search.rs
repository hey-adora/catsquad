use crate::{Db, XTimestamp, join_str};
use catsquad_log::prelude::*;
use catsquad_shared::{Order, TimeRange};
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbUserSearchErr {
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

impl Db {
    pub async fn user_search(
        &self,
        time_range_value: u64,
        limit: usize,
        time_range: TimeRange,
        order: Order,
    ) -> Result<Vec<DbUserSearch>, DbUserSearchErr> {
        let pool = &self.db;

        let q_order = match order {
            Order::OneTwoThree => "ASC",
            Order::ThreeTwoOne => "DESC",
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

        let query_str = format!(
            "
             SELECT
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
             FROM users 
             LEFT JOIN files_images
                ON user_pfp_image_hash = image_hash
             {q_where}
             ORDER BY user_created_at {q_order}
             LIMIT $2
            "
        );

        let query = AssertSqlSafe(query_str.clone());

        let result = sqlx::query_as(query)
            .bind(XTimestamp(time_range_value as i64))
            .bind(limit as i64)
            .fetch_all(pool)
            .await;

        debug!("query {query_str}\nresult {result:#?}");

        let users: Vec<DbUserSearch> = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserSearchErr::Db(err));
            }
        };

        Ok(users)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_search() {
    init_log();

    let db = Db::test_db(0, "test_user_search").await;

    for i in 0..10 {
        let email = format!("hey{i}@hey.com");
        let username = format!("hey{i}");
        let invite = db.invite_add(0, email, 10).await.unwrap();
        let _user = db
            .user_add(i, username, "pss", invite.token.clone(), 10, 10)
            .await
            .unwrap();
    }

    let users = db
        .user_search(0, 10, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(users.len(), 10);
    assert_eq!(users[0].username, "hey0");
    assert_eq!(users[9].username, "hey9");

    let users = db
        .user_search(0, 2, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].username, "hey0");
    assert_eq!(users[1].username, "hey1");

    let users = db
        .user_search(1, 2, TimeRange::MoreOrEqual, Order::OneTwoThree)
        .await
        .unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].username, "hey1");
    assert_eq!(users[1].username, "hey2");
}
