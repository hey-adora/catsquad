use crate::{
    Db, DbAuthErr, DbComment, XTimestamp, ,
    query::user_get_by_session_and_roles::{
        DbUserGetBySessionAndRolesErr, DbUserGetBySessionAndRolesSimpleErr, auth_perms,
        auth_username,
    },
};
use catsquad_log::prelude::*;
use catsquad_shared::{
    Order, PostState, ROLE_VALUE_PERMISSION_SEE_HIDDEN, SessionTokenKind, TimeRange,
    USER_STATE_ALL, USER_STATE_SHOW_ON, UserState, contains_perms_or_is_root,
};
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbCommentSearchErr {
    #[error("internal server error")]
    InternalServerErr,

    #[error("unauthorized")]
    Unauthorized(String),

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

fn from_user_get_by_session_and_roles_err(
    value: DbUserGetBySessionAndRolesErr,
) -> DbCommentSearchErr {
    match DbUserGetBySessionAndRolesSimpleErr::from(value) {
        DbUserGetBySessionAndRolesSimpleErr::Unauthorized(err) => {
            DbCommentSearchErr::Unauthorized(err)
        }
        DbUserGetBySessionAndRolesSimpleErr::Db(err) => DbCommentSearchErr::Db(err),
    }
}

fn from_db_auth_err(value: DbAuthErr) -> DbCommentSearchErr {
    match value {
        DbAuthErr::Unauthorized(err) => DbCommentSearchErr::Unauthorized(err),
        DbAuthErr::InternalServerErr => DbCommentSearchErr::InternalServerErr,
    }
}

impl Db {
    pub async fn comment_search(
        &self,
        auth_session_token: SessionTokenKind,
        post_id: i64,
        parent_id: Option<i64>,
        search_time: u64,
        limit: usize,
        range: TimeRange,
        order: Order,
        flatten: bool,
    ) -> Result<Vec<DbComment>, DbCommentSearchErr> {
        let pool = &self.db;

        // TODO figure it out how to avoid transations in read-only quaries

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("comment_search {err}"))?;

        // check auth
        let user = self
            .auth_optional(&mut *tx, auth_session_token, &[] as &[String])
            .await
            .map_err(from_user_get_by_session_and_roles_err)?;

        let user_permissions = auth_perms(&user);
        let user_username = auth_username(&user);

        let can_see_hidden = contains_perms_or_is_root(
            auth_session_token,
            &[ROLE_VALUE_PERMISSION_SEE_HIDDEN],
            &user_permissions,
        );

        // get post
        {
            let query = "SELECT
                                user_username as post_user_username,
                                post_state,
                                user_state
                               FROM posts
                               JOIN users ON post_user_id = user_id
                               WHERE post_id = $1";

            let result = sqlx::query_as(query).bind(post_id).fetch_one(pool).await;

            debug!("query: {query}\nresult: {result:#?}");

            let result = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Ok(Vec::new());
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbCommentSearchErr::Db(err));
                }
            };

            let (post_user_username, post_state, post_user_state): (String, String, String) =
                result;

            // auth
            {
                let post_state = PostState::try_from(post_state)
                    .inspect_err(|err| error!("{err}"))
                    .map_err(|_err| DbCommentSearchErr::InternalServerErr)?;

                let post_user_state = UserState::try_from(post_user_state)
                    .inspect_err(|err| error!("{err}"))
                    .map_err(|_err| DbCommentSearchErr::InternalServerErr)?;

                let post_is_mine = user_username == post_user_username;
                let post_user_is_active = post_user_state.is_active();
                let post_is_active = post_state.is_active();
                let post_is_draft = post_state.is_draft();

                let can_get_comments = !post_is_draft
                    && ((post_is_active && post_user_is_active)
                        || (can_see_hidden || post_is_mine));

                if !can_get_comments {
                    return Err(DbCommentSearchErr::Unauthorized("unauthorized".to_string()));
                }
            }
        }

        // search
        let result = {
            let show_user_state = match can_see_hidden {
                true => USER_STATE_ALL,
                false => USER_STATE_SHOW_ON,
            };

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

            let q_parent = match (&parent_id, flatten) {
                (Some(_), true) => "AND $2 = ANY(comment_parents)",
                (Some(_), false) => "AND comment_parents[array_length(comment_parents, 1)] = $2",
                (None, true) => "",
                (None, false) => "AND array_length(comment_parents, 1) IS NULL",
            };

            let query_str = format!(
                "
                SELECT
                    comment_id,
                    comment_user_id,
                    user_username as comment_user_username,
                    user_pfp_image_hash as comment_user_pfp_hash,
                    coalesce(image_processed, false) as comment_user_pfp_proccesed,
                    comment_post_id,
                    comment_replies_count,
                    comment_parents,
                    comment_text,
                    comment_modified_at,
                    comment_created_at
                FROM comments
                JOIN users ON comment_user_id=user_id AND user_state = ANY($5)
                LEFT JOIN files_images ON user_pfp_image_hash=image_hash
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
                .bind(show_user_state)
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

            result
        };

        tx.commit()
            .await
            .inspect_err(|err| error!("comment_search {err}"))?;

        Ok(result)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_comment_search() {
    use crate::user_add_with_roles;
    use catsquad_shared::PostState;
    use catsquad_shared::UserState;

    init_log();

    let db = Db::test_db(0, "test_comment_search").await;

    let (user, mut user_token) = user_add_with_roles(
        &db,
        0,
        19,
        10,
        "hey",
        "pss",
        "hey@heyadora.com",
        Vec::<String>::new(),
    )
    .await;

    let (user2, user2_token) = user_add_with_roles(
        &db,
        0,
        19,
        10,
        "hey2",
        "pss",
        "hey2@heyadora.com",
        Vec::<String>::new(),
    )
    .await;

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
            user_token,
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

    // assert post state permisions
    {
        let get_comments_fn = async |token: SessionTokenKind| {
            db.comment_search(
                token,
                post0.id,
                None,
                0,
                10,
                TimeRange::MoreOrEqual,
                Order::ThreeTwoOne,
                false,
            )
            .await
        };

        // TODO add tests for draft

        // post active - user active
        {
            let comments = get_comments_fn(user_token).await.unwrap();
            assert_eq!(comments.len(), 1);

            let comments = get_comments_fn(user2_token).await.unwrap();
            assert_eq!(comments.len(), 1);

            let comments = get_comments_fn(SessionTokenKind::None).await.unwrap();
            assert_eq!(comments.len(), 1);

            let comments = get_comments_fn(SessionTokenKind::Root).await.unwrap();
            assert_eq!(comments.len(), 1);
        }

        // post hidden - user active
        {
            db.post_update_state(0, user.username.clone(), post0.id, PostState::Hidden)
                .await
                .unwrap();

            let comments = get_comments_fn(user_token).await.unwrap();
            assert_eq!(comments.len(), 1);

            let result = get_comments_fn(user2_token).await;
            assert!(matches!(result, Err(DbCommentSearchErr::Unauthorized(_))));

            let result = get_comments_fn(SessionTokenKind::None).await;
            assert!(matches!(result, Err(DbCommentSearchErr::Unauthorized(_))));

            let result = get_comments_fn(SessionTokenKind::Root).await;
            assert!(matches!(result, Ok(_)));
        }

        // TODO must return unauthorized when psot author is hidden
        // TODO must only hide comments of suspended users when they comment on other posts
        // post active - user hidden
        {
            db.post_update_state(0, user.username.clone(), post0.id, PostState::Active)
                .await
                .unwrap();

            db.user_update_state(
                async move |_| String::new(),
                0,
                SessionTokenKind::Root,
                vec!["hey".to_string()],
                UserState::Suspended,
            )
            .await
            .unwrap();

            let result = get_comments_fn(user_token).await;
            assert!(matches!(result, Err(DbCommentSearchErr::Unauthorized(_))));

            let result = get_comments_fn(user2_token).await;
            assert!(matches!(result, Err(DbCommentSearchErr::Unauthorized(_))));

            let result = get_comments_fn(SessionTokenKind::None).await;
            assert!(matches!(result, Err(DbCommentSearchErr::Unauthorized(_))));

            let result = get_comments_fn(SessionTokenKind::Root).await.unwrap();
            assert_eq!(result.len(), 1);
        }

        // post hidden - user hidden - perms_see_hidden
        {
            db.post_update_state(0, user.username.clone(), post0.id, PostState::Hidden)
                .await
                .unwrap();

            db.role_update_value(0, user2.roles[0], vec![ROLE_VALUE_PERMISSION_SEE_HIDDEN])
                .await
                .unwrap();

            let result = get_comments_fn(user_token).await;
            assert!(matches!(result, Err(DbCommentSearchErr::Unauthorized(_))));

            let result = get_comments_fn(user2_token).await.unwrap();
            assert_eq!(result.len(), 1);

            let result = get_comments_fn(SessionTokenKind::None).await;
            assert!(matches!(result, Err(DbCommentSearchErr::Unauthorized(_))));

            let result = get_comments_fn(SessionTokenKind::Root).await.unwrap();
            assert_eq!(result.len(), 1);

            db.post_update_state(0, user.username.clone(), post0.id, PostState::Active)
                .await
                .unwrap();

            db.user_update_state(
                async move |_| String::new(),
                0,
                SessionTokenKind::Root,
                vec!["hey".to_string()],
                UserState::Active,
            )
            .await
            .unwrap();

            let session = db.session_add(0, "hey@heyadora.com").await.unwrap();
            user_token = SessionTokenKind::User(session.token);
        }
    }

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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
            user_token,
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
}
