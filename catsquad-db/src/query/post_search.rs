use catsquad_log::prelude::*;
use catsquad_shared::{Order, PostState, TimeRange};
use surrealdb::types::{RecordId, RecordIdKey};

use crate::{
    Db, DbPost, DbPostFile, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils,
    create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostSearchErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_search(
        &self,
        state: PostState,
        tags: impl Into<String>,
        user: impl Into<String>,
        time_range: u128,
        limit: usize,
        range: TimeRange,
        order: Order,
    ) -> Result<Vec<DbPost>, DbPostSearchErr> {
        let tags = tags.into();
        let user = user.into();

        let tags = tags.to_lowercase();
        let tags = tags.split_whitespace();
        let tags = tags.map(|v| v.to_string()).collect::<Vec<String>>();

        let q_tags = if tags.len() > 0 {
            "tags CONTAINSALL $tags"
        } else {
            ""
        };

        let q_user = if !user.is_empty() {
            "user = (SELECT id FROM ONLY user WHERE username = $user).id"
        } else {
            ""
        };

        let q_time_after = match range {
            TimeRange::None => "",
            TimeRange::Less => "created_at < $time_range",
            TimeRange::LessOrEqual => "created_at <= $time_range",
            TimeRange::More => "created_at > $time_range",
            TimeRange::MoreOrEqual => "created_at >= $time_range",
        };

        let q_order = match order {
            Order::OneTwoThree => "ASC",
            Order::ThreeTwoOne => "DESC",
        };

        let q_state = "state = $post_state";

        let filters = [q_tags, q_time_after, q_user, q_state];
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
            q_where.push_str(q);

            // let next_is_empty = iter.peek().map(|v| v.is_empty()).unwrap_or(true);
            let next_is_empty = iter.peek();
            if next_is_empty.is_none() {
                trace!("q break2");
                break;
            }
            q_where.push_str(" AND ");
        }

        let query = format!(
            "
                SELECT *, user.* FROM post WHERE 
                    {q_where}   
                    ORDER BY created_at {q_order}
                    LIMIT $get_limit;
                "
        );

        trace!("about to run {query}");

        self.db
            .query(query)
            .bind(("get_limit", limit))
            .bind(("time_range", time_range))
            .bind(("post_state", state.to_string()))
            .bind(("tags", tags))
            .bind(("user", user))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbPostSearchErr::Db(err)
                }
            })
            .and_then_take_all(0)
    }
}

#[tokio::test]
async fn test_post_search() {
    init_log();

    let db = Db::mem(0).await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.id.key.clone(), 10, 10)
        .await
        .unwrap();

    let post0 = db
        .post_add(1, user.id.clone(), "1", "description", "one two three")
        .await
        .unwrap();
    let post0 = db
        .post_update_state(1, user.id.clone(), post0.id.key, PostState::Active)
        .await
        .unwrap();
    let post1 = db
        .post_add(2, user.id.clone(), "2", "description", "one two")
        .await
        .unwrap();
    let post1 = db
        .post_update_state(2, user.id.clone(), post1.id.key, PostState::Active)
        .await
        .unwrap();
    let post2 = db
        .post_add(3, user.id.clone(), "3", "description", "one")
        .await
        .unwrap();
    let post2 = db
        .post_update_state(3, user.id.clone(), post2.id.key, PostState::Active)
        .await
        .unwrap();
    let post9 = db
        .post_add(0, user.id.clone(), "9", "description9", "one two three 9")
        .await
        .unwrap();

    let result = db
        .post_search(
            PostState::Active,
            "",
            "",
            0,
            4,
            TimeRange::MoreOrEqual,
            Order::ThreeTwoOne,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 3);

    {
        let result = db
            .post_search(
                PostState::Active,
                " three  two     ",
                "hey",
                3,
                3,
                TimeRange::LessOrEqual,
                Order::ThreeTwoOne,
            )
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(&result[0].title, "1");

        let result = db
            .post_search(
                PostState::Active,
                " three  two     ",
                "hey2",
                3,
                3,
                TimeRange::LessOrEqual,
                Order::ThreeTwoOne,
            )
            .await
            .unwrap();
        assert_eq!(result.len(), 0);
    }

    let result = db
        .post_search(
            PostState::Active,
            " three  two     ",
            String::new(),
            3,
            3,
            TimeRange::LessOrEqual,
            Order::ThreeTwoOne,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "1");

    let result = db
        .post_search(
            PostState::Active,
            "three two",
            String::new(),
            3,
            3,
            TimeRange::LessOrEqual,
            Order::ThreeTwoOne,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "1");

    let result = db
        .post_search(
            PostState::Active,
            "two",
            String::new(),
            3,
            3,
            TimeRange::LessOrEqual,
            Order::ThreeTwoOne,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "2");
    assert_eq!(&result[1].title, "1");

    let result = db
        .post_search(
            PostState::Active,
            "two",
            String::new(),
            3,
            3,
            TimeRange::LessOrEqual,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = db
        .post_search(
            PostState::Active,
            "two",
            String::new(),
            1,
            3,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = db
        .post_search(
            PostState::Active,
            "two",
            String::new(),
            0,
            3,
            TimeRange::None,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = db
        .post_search(
            PostState::Active,
            "two",
            String::new(),
            3,
            3,
            TimeRange::Less,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = db
        .post_search(
            PostState::Active,
            "two",
            String::new(),
            2,
            3,
            TimeRange::Less,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "1");

    let result = db
        .post_search(
            PostState::Active,
            "two",
            String::new(),
            1,
            3,
            TimeRange::More,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "2");
}
