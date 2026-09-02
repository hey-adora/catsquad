use crate::{Db, DbComment, XTimestamp, join_str};
use catsquad_log::prelude::*;
use catsquad_shared::{Order, TimeRange};
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbCommentSearchErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn comment_search(
        &self,
        post_id: i64,
        parent_id: Option<i64>,
        search_time: u64,
        limit: usize,
        range: TimeRange,
        order: Order,
        flatten: bool,
    ) -> Result<Vec<DbComment>, DbCommentSearchErr> {
        let pool = &self.db;

        let q_order = match order {
            Order::OneTwoThree => "ASC",
            Order::ThreeTwoOne => "DESC",
        };

        let q_time_after = match range {
            TimeRange::None => "",
            TimeRange::Less => "AND comment_created_at < $1",
            TimeRange::LessOrEqual => "AND comment_created_at <= $1",
            TimeRange::More => "AND comment_created_at > $1",
            TimeRange::MoreOrEqual => "AND comment_created_at >= $1",
        };
        // .to_string();

        let q_parent = match (&parent_id, flatten) {
            (Some(_), true) => "AND $2 = ANY(comment_parents)",
            (Some(_), false) => "AND comment_parents[array_length(comment_parents, 1)] = $2",
            (None, true) => "",
            (None, false) => "AND array_length(comment_parents, 1) IS NULL",
        };

        let query_str = format!(
            "
            SELECT * FROM comments
                WHERE comment_post_id = $4 {q_time_after} {q_parent}
                ORDER BY comment_created_at {q_order}
                LIMIT $3
        "
        );

        let query = AssertSqlSafe(query_str.clone());

        let result = sqlx::query_as(query)
            .bind(XTimestamp(search_time as i64))
            .bind(parent_id.unwrap_or_default())
            .bind(limit as i64)
            .bind(post_id)
            .fetch_all(pool)
            .await;

        debug!(
            "query: {query_str}\n $1={}, $2={}, $3={}, $4={}\nresult: {result:#?}",
            search_time as i64,
            parent_id.unwrap_or_default(),
            limit as i64,
            post_id
        );

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbCommentSearchErr::Db(err));
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_comment_search() {
    use catsquad_shared::PostState;

    init_log();

    let db = Db::test_db(0, "test_comment_search").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();
    let user = db
        .user_add(0, "hey", "hey", invite1.token, 10, 10)
        .await
        .unwrap();

    let post0 = db
        .post_add(
            1,
            user.username.clone(),
            "1",
            "description",
            "one two three",
        )
        .await
        .unwrap();
    db.post_update_state(0, user.username.clone(), post0.id, PostState::Active)
        .await
        .unwrap();

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            None,
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::ThreeTwoOne,
            false,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 0);

    let comment0 = db
        .comment_add(0, user.username.clone(), post0.id, None, "one0")
        .await
        .unwrap();

    let comment0_r0 = db
        .comment_add(
            1,
            user.username.clone(),
            post0.id,
            Some(comment0.id),
            "one0r0",
        )
        .await
        .unwrap();

    let comment0_r1 = db
        .comment_add(
            2,
            user.username.clone(),
            post0.id,
            Some(comment0_r0.id),
            "one0r1",
        )
        .await
        .unwrap();

    let comment0_r2 = db
        .comment_add(
            3,
            user.username.clone(),
            post0.id,
            Some(comment0_r1.id),
            "one0r2",
        )
        .await
        .unwrap();

    let comment1 = db
        .comment_add(4, user.username.clone(), post0.id, None, "one1")
        .await
        .unwrap();

    let _comment1_r0 = db
        .comment_add(
            1,
            user.username.clone(),
            post0.id,
            Some(comment1.id),
            "one1r0",
        )
        .await
        .unwrap();

    let comment2 = db
        .comment_add(5, user.username.clone(), post0.id, None, "one2")
        .await
        .unwrap();

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            None,
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::ThreeTwoOne,
            false,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 3);
    assert_eq!(comments[0].text, "one2");
    assert_eq!(comments[1].text, "one1");
    assert_eq!(comments[2].text, "one0");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            None,
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
            false,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 3);
    assert_eq!(comments[2].text, "one2");
    assert_eq!(comments[1].text, "one1");
    assert_eq!(comments[0].text, "one0");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0.id),
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
            false,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text, "one0r0");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0_r0.id),
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
            false,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text, "one0r1");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0_r1.id),
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
            false,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text, "one0r2");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0_r2.id),
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
            false,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 0);

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0.id),
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
            true,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 3);
    assert_eq!(comments[0].text, "one0r0");
    assert_eq!(comments[1].text, "one0r1");
    assert_eq!(comments[2].text, "one0r2");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0.id),
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::ThreeTwoOne,
            true,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 3);
    assert_eq!(comments[2].text, "one0r0");
    assert_eq!(comments[1].text, "one0r1");
    assert_eq!(comments[0].text, "one0r2");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0.id),
            2,
            10,
            TimeRange::MoreOrEqual,
            Order::ThreeTwoOne,
            true,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[1].text, "one0r1");
    assert_eq!(comments[0].text, "one0r2");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0.id),
            2,
            10,
            TimeRange::More,
            Order::ThreeTwoOne,
            true,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text, "one0r2");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0.id),
            2,
            10,
            TimeRange::LessOrEqual,
            Order::ThreeTwoOne,
            true,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[1].text, "one0r0");
    assert_eq!(comments[0].text, "one0r1");

    let comments = db
        .comment_search(
            // 0,
            post0.id,
            Some(comment0.id),
            2,
            10,
            TimeRange::Less,
            Order::ThreeTwoOne,
            true,
        )
        .await
        .unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text, "one0r0");

    // TODO add tests for post.show, dont show comments publicly when post is hidden, only owner
}
