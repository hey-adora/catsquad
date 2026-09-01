use crate::{Db, DbPost, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::{Order, PostState, TimeRange};
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbPostSearchErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
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
    ) -> Result<Vec<DbPost>, DbPostSearchErr> {
        let pool = &self.db;
        let user_username = user.into();
        let mut bind_index = 4_usize;

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
        let tags_len = tags.len();

        // itertool
        // position(' two ' in post_tags) > 0
        let q_tags = if tags.len() > 0 {
            let mut output = String::new();

            for i in 0..tags_len {
                output += &format!("position(${bind_index} in post_tags) > 0");
                if i != tags_len - 1 {
                    output += " AND ";
                }

                bind_index += 1;
            }

            output
            // "LIKE '%' || LOWER($4) || '%'"
        } else {
            "".to_string()
        };

        let q_user = if !user_username.is_empty() {
            format!("user = ${bind_index}")
        } else {
            "".to_string()
        };

        let filters = [q_tags, q_time_after, q_user, q_state];
        // let filters = [q_time_after, q_state];
        let mut q_where = String::new();
        let mut iter = filters.into_iter().peekable();

        loop {
            let Some(q) = iter.next() else {
                trace!("q break");
                break;
            };
            trace!("reading q {q}");
            if q.is_empty() {
                trace!("q continue");
                continue;
            }
            q_where.push_str(&q);

            // let next_is_empty = iter.peek().map(|v| v.is_empty()).unwrap_or(true);
            let next_is_empty = iter.peek();
            if next_is_empty.is_none() {
                trace!("q break2");
                break;
            }
            q_where.push_str(" AND ");
        }

        let query_str = format!(
            "
            SELECT * FROM posts
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

    let result = search("", "", 0, 4, 4, true).await;
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].title, "3");
    assert_eq!(result[1].title, "2");
    assert_eq!(result[2].title, "1");

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
