use crate::{Db, DbPost, XTimestamp, if_empty, join_str};
use catsquad_log::prelude::*;
use catsquad_shared::{Order, PostState, TimeRange};
use sqlx::AssertSqlSafe;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbPostSearch {
    #[sqlx(rename = "post_id")]
    pub id: i64,
    #[sqlx(rename = "post_user_username")]
    pub user_username: String,
    #[sqlx(rename = "post_state")]
    pub state: String, // TODO optimize to be enum
    #[sqlx(rename = "post_title")]
    pub title: String,
    #[sqlx(rename = "post_description")]
    pub description: String,
    #[sqlx(rename = "post_tags")]
    pub tags: String,
    #[sqlx(rename = "post_likes_count")]
    #[sqlx(try_from = "i64")]
    pub likes_count: u32,
    #[sqlx(rename = "post_size_bytes")]
    #[sqlx(try_from = "i64")]
    pub size_bytes: u32,
    #[sqlx(rename = "post_image_width")]
    #[sqlx(try_from = "i64")]
    pub image_width: u32,
    #[sqlx(rename = "post_image_height")]
    #[sqlx(try_from = "i64")]
    pub image_height: u32,
    #[sqlx(rename = "post_image_extension")]
    pub image_extension: String,
    #[sqlx(rename = "post_image_hash")]
    pub image_hash: i64,
    // #[sqlx(rename = "post_images_hashes")]
    // pub images_hashes: Vec<i64>,
    #[sqlx(rename = "post_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "post_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DbPostSearchErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

pub fn split_tags(tags: impl Into<String>) -> Vec<String> {
    let tags = tags.into();
    let tags = tags.to_lowercase();
    let tags = tags.split_whitespace();
    let tags = tags
        .map(|v| {
            let mut tag = String::new();
            tag.push(' ');
            tag.push_str(v);
            tag.push(' ');
            tag
        })
        .collect::<Vec<String>>();
    tags
}

impl Db {
    pub async fn post_search(
        &self,
        state: PostState,
        tags: impl Into<String>,
        user: impl Into<String>,
        search_time: u64,
        limit: usize,
        range: TimeRange,
        order: Order,
    ) -> Result<Vec<DbPostSearch>, DbPostSearchErr> {
        let pool = &self.db;
        let user_username = user.into();
        let mut bind_index = 3_usize;

        let q_order = match order {
            Order::OneTwoThree => "ASC",
            Order::ThreeTwoOne => "DESC",
        };

        let q_time_after = match range {
            TimeRange::None => "",
            TimeRange::Less => "post_created_at < $1",
            TimeRange::LessOrEqual => "post_created_at <= $1",
            TimeRange::More => "post_created_at > $1",
            TimeRange::MoreOrEqual => "post_created_at >= $1",
        }
        .to_string();

        let q_state = "post_state = $2".to_string();

        let tags = split_tags(tags);

        let q_tags = join_str(0..tags.len(), " AND ", |_| {
            bind_index += 1;
            format!("position(${bind_index} in post_tags) > 0")
        });

        let q_user = if_empty(&user_username, || {
            bind_index += 1;
            format!("user = ${bind_index}")
        });

        let filters = [q_tags, q_time_after, q_user, q_state];
        let q_where = join_str(filters, " AND ", |v| v);

        let query_str = format!(
            "
            SELECT
                post_id,
                post_user_username,
                post_state,
                post_title,
                post_description,
                post_tags,
                post_likes_count,
                post_size_bytes,
                coalesce(image_width, 0) as post_image_width,
                coalesce(image_height, 0) as post_image_height,
                coalesce(image_extension, '') as post_image_extension,
                coalesce(image_hash, 0) as post_image_hash,
                post_modified_at,
                post_created_at

                FROM posts
                LEFT JOIN files_images ON post_images_hashes[1]=image_hash
                WHERE {q_where}
                ORDER BY post_created_at {q_order}
                LIMIT $3
        "
        );

        let query = AssertSqlSafe(query_str.clone());

        let mut builder = sqlx::query_as(query)
            .bind(XTimestamp(search_time as i64))
            .bind(state.as_str())
            .bind(limit as i64);

        for tag in tags {
            builder = builder.bind(tag);
        }

        if !user_username.is_empty() {
            builder = builder.bind(user_username);
        }

        let result = builder.fetch_all(pool).await;

        trace!("query {query_str} result {result:#?}");

        let posts = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostSearchErr::Db(err));
            }
        };

        Ok(posts)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_search() {
    use crate::DbUser;

    init_log();

    let db = Db::test_db(0, "test_post_search").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let add_post = async |time: u64, user: &DbUser, title: &str, description: &str, tags: &str| {
        db.post_add(time, user.username.clone(), title, description, tags)
            .await
            .unwrap()
    };

    let add_post_and_activate =
        async |time: u64, user: &DbUser, title: &str, description: &str, tags: &str| {
            let post0 = db
                .post_add(time, user.username.clone(), title, description, tags)
                .await
                .unwrap();
            db.post_update_state(time, user.username.clone(), post0.id, PostState::Active)
                .await
                .unwrap();
            post0
        };

    let search =
        async |tags: &str, user: &str, time: u64, limit: usize, time_range: u8, order: bool| {
            let result = db
                .post_search(
                    PostState::Active,
                    tags,
                    user,
                    time,
                    limit,
                    TimeRange::from(time_range),
                    Order::from(order),
                )
                .await
                .unwrap();

            result
        };

    let post0 = add_post_and_activate(1, &user, "1", "description", "one two three").await;
    let post1 = add_post_and_activate(2, &user, "2", "description", "one two").await;
    let post2 = add_post_and_activate(3, &user, "3", "description", "one").await;
    let post9 = add_post(4, &user, "9", "description9", "one two three 9").await;

    db.post_update_file_add(0, user.username, post0.id, 10, 666, "jpg", 10, 15)
        .await
        .unwrap();

    let result = search("", "", 0, 4, 4, true).await;
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].title, "3");
    assert_eq!(result[0].image_hash, 0);
    assert_eq!(result[1].title, "2");
    assert_eq!(result[1].image_hash, 0);
    assert_eq!(result[2].title, "1");
    assert_eq!(result[2].image_width, 10);
    assert_eq!(result[2].image_height, 15);
    assert_eq!(result[2].image_extension, "jpg");
    assert_eq!(result[2].image_hash, 666);

    let result = search(" three  two     ", "hey", 3, 3, 2, true).await;
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "1");

    let result = search(" three  two     ", "hey2", 3, 3, 2, true).await;
    assert_eq!(result.len(), 0);

    let result = search("three two", "", 3, 3, 2, true).await;
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "1");

    let result = search("two", "", 3, 3, 2, true).await;
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "2");
    assert_eq!(&result[1].title, "1");

    let result = search("two", "", 3, 3, 2, false).await;
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = search("two", "", 1, 3, 4, false).await;
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = search("two", "", 0, 3, 0, false).await;
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = search("two", "", 3, 3, 1, false).await;
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = search("two", "", 3, 1, 1, false).await;
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "1");

    let result = search("two", "", 2, 3, 1, false).await;
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "1");

    let result = search("two", "", 1, 3, 3, false).await;
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "2");

    let result = search("tw", "", 1, 3, 3, false).await;
    assert_eq!(result.len(), 0);
}
