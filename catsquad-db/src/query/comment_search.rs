use catsquad_log::prelude::*;
use catsquad_shared::{Order, TimeRange};
use surrealdb::types::RecordIdKey;

use crate::{
    Db, DbComment, SurrealCheckUtils, SurrealSerializeUtils, create_comment_id, create_post_id,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbCommentSearchErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn comment_search(
        &self,
        // time: u128,
        post_key: impl Into<RecordIdKey>,
        parent_key: Option<impl Into<RecordIdKey>>,
        time_range: u128,
        limit: usize,
        range: TimeRange,
        order: Order,
        flatten: bool,
    ) -> Result<Vec<DbComment>, DbCommentSearchErr> {
        let post_id = create_post_id(post_key);
        let parent_id = parent_key.map(|v| create_comment_id(v.into()));

        let q_time_after = match range {
            TimeRange::None => "",
            TimeRange::Less => "AND created_at < $time_range",
            TimeRange::LessOrEqual => "AND created_at <= $time_range",
            TimeRange::More => "AND created_at > $time_range",
            TimeRange::MoreOrEqual => "AND created_at >= $time_range",
        };

        let q_order = match order {
            Order::OneTwoThree => "ASC",
            Order::ThreeTwoOne => "DESC",
        };
        let q_parent = match (&parent_id, flatten) {
            (Some(_), true) => "AND parent.find($parent_id)",
            (Some(_), false) => "AND parent.last() = $parent_id",
            (None, true) => "",
            (None, false) => "AND parent.len() = 0",
        };

        let query = format!(
            "
            SELECT *, user.* FROM comment WHERE
                    post = $post_id
                    {q_time_after}
                    {q_parent}
                    ORDER BY created_at {q_order}
                    LIMIT $comment_limit
        "
        );

        trace!("about to run {query}");

        self.db
            .query(query)
            // .bind(("time", time))
            .bind(("time_range", time_range))
            .bind(("parent_id", parent_id))
            .bind(("post_id", post_id))
            .bind(("comment_limit", limit))
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbCommentSearchErr::Db(err)
                }
            })
            .and_then_take_all(0)
    }
}

#[tokio::test]
async fn test_comment_search() {
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

    let comments = db
        .comment_search(
            // 0,
            post0.id.key.clone(),
            None::<RecordIdKey>,
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
        .comment_add(
            0,
            user.id.clone(),
            post0.id.key.clone(),
            None::<RecordIdKey>,
            "one0",
        )
        .await
        .unwrap();

    let comment0_r0 = db
        .comment_add(
            1,
            user.id.clone(),
            post0.id.key.clone(),
            Some(comment0.id.key.clone()),
            "one0r0",
        )
        .await
        .unwrap();

    let comment0_r1 = db
        .comment_add(
            2,
            user.id.clone(),
            post0.id.key.clone(),
            Some(comment0_r0.id.key.clone()),
            "one0r1",
        )
        .await
        .unwrap();

    let comment0_r2 = db
        .comment_add(
            3,
            user.id.clone(),
            post0.id.key.clone(),
            Some(comment0_r1.id.key.clone()),
            "one0r2",
        )
        .await
        .unwrap();

    let comment1 = db
        .comment_add(
            4,
            user.id.clone(),
            post0.id.key.clone(),
            None::<RecordIdKey>,
            "one1",
        )
        .await
        .unwrap();

    let _comment1_r0 = db
        .comment_add(
            1,
            user.id.clone(),
            post0.id.key.clone(),
            Some(comment1.id.key.clone()),
            "one1r0",
        )
        .await
        .unwrap();

    let comment2 = db
        .comment_add(
            5,
            user.id.clone(),
            post0.id.key.clone(),
            None::<RecordIdKey>,
            "one2",
        )
        .await
        .unwrap();

    let comments = db
        .comment_search(
            // 0,
            post0.id.key.clone(),
            None::<RecordIdKey>,
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
            post0.id.key.clone(),
            None::<RecordIdKey>,
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
            post0.id.key.clone(),
            Some(comment0.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0_r0.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0_r1.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0_r2.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0.id.key.clone()),
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
            post0.id.key.clone(),
            Some(comment0.id.key.clone()),
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
