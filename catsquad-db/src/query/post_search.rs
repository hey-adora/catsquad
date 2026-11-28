use crate::{
    Db, DbUserGetBySessionAndRolesErr, DbUserGetBySessionAndRolesSimpleErr, XTimestamp, auth_perms,
    auth_username, join_str,
};
use catsquad_log::prelude::*;
use catsquad_shared::{
    Order, PostState, ROLE_VALUE_PERMISSION_SEE_HIDDEN, RoleValue, SessionTokenKind, TimeRange,
    UserState, contains_perms_or_is_root,
};
use sqlx::AssertSqlSafe;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbPostSearch {
    #[sqlx(rename = "post_id")]
    pub id: i64,
    #[sqlx(rename = "post_user_id")]
    pub user_id: i64,
    #[sqlx(rename = "post_user_username")]
    pub user_username: String,
    #[sqlx(rename = "post_user_state")]
    pub user_state: String,
    #[sqlx(rename = "post_user_pfp_hash")]
    pub user_pfp_hash: i64,
    #[sqlx(rename = "post_user_pfp_proccesed")]
    pub user_pfp_proccesed: bool,
    #[sqlx(rename = "post_state")]
    pub state: String, // TODO optimize to be enum
    #[sqlx(rename = "post_title")]
    pub title: String,
    #[sqlx(rename = "post_description")]
    pub description: String,
    #[sqlx(rename = "post_tags")]
    pub tags: String,
    #[sqlx(rename = "post_liked")]
    pub liked: bool,
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

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("internal server error")]
    InternalServerErr,
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

fn from_user_get_by_session_and_roles_err(value: DbUserGetBySessionAndRolesErr) -> DbPostSearchErr {
    match DbUserGetBySessionAndRolesSimpleErr::from(value) {
        DbUserGetBySessionAndRolesSimpleErr::Unauthorized(err) => {
            DbPostSearchErr::Unauthorized(err)
        }
        DbUserGetBySessionAndRolesSimpleErr::Db(err) => DbPostSearchErr::Db(err),
    }
}

impl Db {
    pub async fn post_search(
        &self,
        tags: impl Into<String>,
        auth_session_token: SessionTokenKind,
        author_username: impl Into<String>,
        liked_by_username: impl Into<String>,
        followed_by_username: impl Into<String>,
        search_time: u64, // TODO rename to time_range_value maybe
        limit: usize,
        range: TimeRange,
        order: Order,
    ) -> Result<Vec<DbPostSearch>, DbPostSearchErr> {
        let pool = &self.db;

        let author_username = author_username.into();
        let liked_by_username = liked_by_username.into();
        let followed_by_username = followed_by_username.into();

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_search {err}"))?;

        // check auth
        let user = self
            .auth_optional(&mut *tx, auth_session_token, &[] as &[String])
            .await
            .map_err(from_user_get_by_session_and_roles_err)?;

        let user_permissions = auth_perms(&user);
        let user_username = auth_username(&user);
        let can_see_hidden = contains_perms_or_is_root(
            auth_session_token,
            &[RoleValue::PermissionSeeHidden],
            &user_permissions,
        );
        let is_my_posts = !user_username.is_empty() && user_username == author_username;
        let can_see_hidden = can_see_hidden || is_my_posts;

        let is_liked_by_and_user_same =
            !user_username.is_empty() && user_username == liked_by_username;

        // auth
        {
            if !author_username.is_empty() && !can_see_hidden {
                let query = "SELECT user_state FROM users WHERE user_username = $1";
                let result = sqlx::query_as(query)
                    .bind(&author_username)
                    .fetch_one(&mut *tx)
                    .await;

                let (author_user_state,): (String,) = match result {
                    Ok(v) => v,
                    Err(err) => {
                        error!("unexpected db error {err}");
                        return Err(DbPostSearchErr::Db(err));
                    }
                };

                let author_user_state = match UserState::try_from(author_user_state.as_str()) {
                    Ok(v) => v,
                    Err(_err) => {
                        error!(
                            "something exploded, user {} has invalid state {}",
                            author_username, author_user_state
                        );
                        return Err(DbPostSearchErr::InternalServerErr);
                    }
                };

                if author_user_state.is_inactive() {
                    return Err(DbPostSearchErr::Unauthorized("suspended".to_string()));
                }
            }
        }

        // order

        let q_order = match order {
            Order::OneTwoThree => "ASC",
            Order::ThreeTwoOne => "DESC",
        };

        // filter - time

        let q_time_after = match range {
            TimeRange::None => "",
            TimeRange::Less => "post_created_at < $1",
            TimeRange::LessOrEqual => "post_created_at <= $1",
            TimeRange::More => "post_created_at > $1",
            TimeRange::MoreOrEqual => "post_created_at >= $1",
        };

        // filter - tags

        let tags = split_tags(tags);

        let q_tags = join_str(1..=tags.len(), " AND ", |i| {
            format!("position(($4)[{i}] in post_tags) > 0")
        });

        // filter - liked by user

        // TODO investigate, somehow works with field `user`
        // TODO figure out what the hell that TODO above means

        let (q_user_filter,) = match author_username.is_empty() {
            true => ("",),
            false => ("post_user.user_username = $5",),
        };

        // filter - state

        let q_state = match can_see_hidden {
            true => "post_state != $2",
            false => "post_state = $10 AND post_user.user_state = $9",
        };

        //

        let (q_liked_by_user_join1, q_liked_by_user_join2) = match liked_by_username.is_empty() {
            true => ("", ""),
            false => (
                "JOIN users liked_by_user ON $6=liked_by_user.user_username",
                "INNER JOIN posts_likes l1 ON post_id=l1.post_like_post_id AND liked_by_user.user_id=l1.post_like_user_id",
            ),
        };

        // field - liked by me

        let (q_liked_by_me_join1, q_liked_by_me_join2) =
            match is_liked_by_and_user_same || user_username.is_empty() {
                true => ("", ""),
                false => (
                    "JOIN users my_user ON $7=my_user.user_username",
                    "LEFT JOIN posts_likes l2 ON
                        post_id=l2.post_like_post_id
                        AND my_user.user_id = l2.post_like_user_id",
                ),
            };

        let q_liked_by_me_field = 'a: {
            if is_liked_by_and_user_same {
                break 'a "true as post_liked";
            }

            if !user_username.is_empty() {
                break 'a "coalesce(l2.post_like_post_id > 0, false) as post_liked";
            }

            "false as post_liked"
        };

        // filter - followed by

        let (q_followed_by_user_join1, q_followed_by_user_join2) = match followed_by_username
            .is_empty()
        {
            true => ("", ""),
            false => (
                "JOIN users followed_user ON $8=followed_user.user_username",
                "JOIN follow fo1 ON fo1.follower_user_id=followed_user.user_id AND post_user.user_id=fo1.following_user_id",
            ),
        };

        let filters = [q_tags.as_str(), q_time_after, q_user_filter, q_state];
        let q_where = join_str(filters, " AND ", |v| v);
        let q_where = if !q_where.is_empty() {
            format!("WHERE {q_where}")
        } else {
            String::new()
        };

        // post_user_pfp_proccesed
        let query_str = format!(
            "
            SELECT
                post_id,
                post_user_id,
                post_user.user_username as post_user_username,
                post_user.user_state as post_user_state,
                post_user.user_pfp_image_hash as post_user_pfp_hash,
                coalesce(f2.image_processed, false) as post_user_pfp_proccesed,
                post_state,
                post_title,
                post_description,
                post_tags,
                {q_liked_by_me_field},
                post_likes_count,
                post_size_bytes,
                coalesce(f1.image_width, 0) as post_image_width,
                coalesce(f1.image_height, 0) as post_image_height,
                coalesce(f1.image_extension, '') as post_image_extension,
                coalesce(f1.image_hash, 0) as post_image_hash,
                post_modified_at,
                post_created_at

                FROM posts

                JOIN users post_user ON post_user_id=post_user.user_id
                {q_liked_by_user_join1}
                {q_liked_by_me_join1}
                {q_followed_by_user_join1}

                LEFT JOIN files_images f1 ON post_images_hashes[1]=f1.image_hash
                LEFT JOIN files_images f2 ON post_user.user_pfp_image_hash=f2.image_hash

                {q_liked_by_user_join2}
                {q_liked_by_me_join2}
                {q_followed_by_user_join2}

                {q_where}

                ORDER BY post_created_at {q_order}
                LIMIT $3
        "
        );

        let query = AssertSqlSafe(query_str.clone());

        let builder = sqlx::query_as(query)
            .bind(XTimestamp(search_time as i64))
            .bind(PostState::Draft.as_str())
            .bind(limit as i64)
            .bind(&tags)
            .bind(&author_username)
            .bind(&liked_by_username)
            .bind(&user_username)
            .bind(&followed_by_username)
            .bind(UserState::Active.as_str())
            .bind(PostState::Active.as_str());

        let result = builder.fetch_all(&mut *tx).await;

        trace!(
            "
            query {query_str}
            $1 = {search_time}
            $2 = {}
            $3 = {limit}
            $4 = {tags:?}
            $5 = {author_username}
            $6 = {liked_by_username}
            $7 = {user_username}
            $8 = {followed_by_username}
            $9 = {}
            $10 = {}
            result {result:#?}",
            PostState::Draft.as_str(),
            UserState::Active.as_str(),
            PostState::Active.as_str()
        );

        let posts = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostSearchErr::Db(err));
            }
        };

        tx.commit()
            .await
            .inspect_err(|err| error!("post_search {err}"))?;

        Ok(posts)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_search() {
    use crate::{DbUser, user_add_with_roles};
    use catsquad_shared::UUID_ZERO;

    init_log();

    let db = Db::test_db(0, "test_post_search").await;

    let (user1, mut session1) = user_add_with_roles(
        &db,
        0,
        10,
        10,
        "hey",
        "pss",
        "hey@heyadora.com",
        vec![] as Vec<String>,
    )
    .await;

    let (user2, session2) = user_add_with_roles(
        &db,
        0,
        10,
        10,
        "hey2",
        "pss",
        "hey2@heyadora.com",
        vec![] as Vec<String>,
    )
    .await;

    let (user3, session3) = user_add_with_roles(
        &db,
        0,
        10,
        10,
        "hey3",
        "pss",
        "hey3@heyadora.com",
        vec![] as Vec<String>,
    )
    .await;

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

    let search = async |tags: &str,
                        author_user: &str,
                        time: u64,
                        limit: usize,
                        time_range: u8,
                        order: bool| {
        let result = db
            .post_search(
                tags,
                SessionTokenKind::None,
                author_user,
                "",
                "",
                time,
                limit,
                TimeRange::from(time_range),
                Order::from(order),
            )
            .await
            .unwrap();

        result
    };

    let post0 = add_post_and_activate(1, &user1, "1", "description", "one two three").await;
    let post1 = add_post_and_activate(2, &user1, "2", "description", "one two").await;
    let post2 = add_post_and_activate(3, &user1, "3", "description", "one").await;
    let post9 = add_post(4, &user1, "9", "description9", "one two three 9").await;

    db.post_like_add(4, "hey2", post2.id).await.unwrap();
    db.post_like_add(4, "hey3", post2.id).await.unwrap();
    db.post_like_add(4, "hey3", post1.id).await.unwrap();

    db.post_update_image_add(0, user1.username, post0.id, 10, 666, "jpg", 10, 15)
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

    let result = search("two", "hey", 0, 3, 0, false).await;
    assert_eq!(result.len(), 2);
    assert_eq!(&result[0].title, "1");
    assert_eq!(&result[1].title, "2");

    let result = db
        .post_search(
            "",
            SessionTokenKind::None,
            "",
            "hey2",
            "",
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "3");
    assert_eq!(result[0].liked, false);

    let result = db
        .post_search(
            "",
            session2,
            "",
            "hey2",
            "",
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "3");
    assert_eq!(result[0].liked, true);

    let result = db
        .post_search(
            "",
            session3,
            "",
            "hey2",
            "",
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "3");
    assert_eq!(result[0].liked, true);
    assert_eq!(result[0].user_pfp_hash, 0);
    assert_eq!(result[0].user_pfp_proccesed, false);

    db.user_update_pfp(0, "hey", async move |_, _| (), 10, 666, "webp", 10, 12)
        .await
        .unwrap();

    let result = db
        .post_search(
            "",
            session1,
            "",
            "hey2",
            "",
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].title, "3");
    assert_eq!(result[0].liked, false);
    assert_eq!(result[0].user_pfp_hash, 666);
    assert_eq!(result[0].user_pfp_proccesed, false);

    db.file_image_update_proccsed(0, 666).await.unwrap();

    let result = db
        .post_search(
            "",
            session2,
            "",
            "",
            "",
            0,
            10,
            TimeRange::MoreOrEqual,
            Order::OneTwoThree,
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(&result[0].title, "1");
    assert_eq!(result[0].liked, false);
    assert_eq!(result[0].user_pfp_hash, 666);
    assert_eq!(result[0].user_pfp_proccesed, true);
    assert_eq!(&result[1].title, "2");
    assert_eq!(result[1].liked, false);
    assert_eq!(&result[2].title, "3");
    assert_eq!(result[2].liked, true);

    // following
    {
        let result = db
            .post_search(
                "",
                session2,
                "",
                "",
                "hey2",
                0,
                10,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 0);

        db.follow_add(0, "hey2", "hey").await.unwrap();

        let result = db
            .post_search(
                "",
                session2,
                "",
                "",
                "hey2",
                0,
                10,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
    }

    // assert error unauthorized
    {
        let post_state = async |post_state: PostState| {
            db.post_update_state(0, "hey".to_string(), post0.id, post_state)
                .await
                .unwrap();
        };

        let user_state = async |user_state: UserState| {
            db.user_update_state(
                async |_| String::new(),
                0,
                SessionTokenKind::Root,
                vec!["hey".to_string()],
                user_state,
            )
            .await
            .unwrap();

            if user_state == UserState::Active {
                let token = db.session_add(0, "hey@heyadora.com").await.unwrap().token;
                return SessionTokenKind::User(token);
            }

            SessionTokenKind::None
        };

        let search = async |token: SessionTokenKind, author_username: &str| {
            db.post_search(
                "",
                token,
                author_username,
                "",
                "",
                0,
                10,
                TimeRange::MoreOrEqual,
                Order::OneTwoThree,
            )
            .await
        };

        let results = async |user1_token: SessionTokenKind, author: &str| {
            let result1 = search(SessionTokenKind::None, author).await;
            let result2 = search(user1_token, author).await;
            let result3 = search(session2, author).await;
            let result4 = search(SessionTokenKind::Root, author).await;

            (result1, result2, result3, result4)
        };

        // home page
        {
            // count posts returned for each type of user
            let results = async || {
                let result = results(session1, "").await;
                [
                    result.0.unwrap().len(),
                    result.1.unwrap().len(),
                    result.2.unwrap().len(),
                    result.3.unwrap().len(),
                ]
            };

            // post0 active - user1 active
            {
                let result = results().await;
                assert_eq!(result, [3, 3, 3, 3]);
            }

            // post0 hidden - user1 active
            {
                post_state(PostState::Hidden).await;

                let result = results().await;
                assert_eq!(result, [2, 2, 2, 3]);
            }

            // post0 active - user1 suspended
            {
                post_state(PostState::Active).await;
                user_state(UserState::Suspended).await;

                let result = results().await;
                assert_eq!(result, [0, 0, 0, 3]);
            }

            // post0 active - user1 suspended - see hidden
            {
                db.role_update_value(
                    SessionTokenKind::Root,
                    0,
                    user2.roles[0],
                    vec![ROLE_VALUE_PERMISSION_SEE_HIDDEN],
                )
                .await
                .unwrap();

                let result = results().await;
                assert_eq!(result, [0, 0, 3, 3]);
            }
        }

        // profile page
        {
            // post0 active - user1 active
            {
                session1 = user_state(UserState::Active).await;

                let result = results(session1, "hey").await;
                assert!(matches!(result, (Ok(_), Ok(_), Ok(_), Ok(_))));
            }

            // post0 active - user1 suspended
            {
                user_state(UserState::Suspended).await;

                let result = results(session1, "hey").await;
                trace!("{result:#?}");
                assert!(matches!(
                    result,
                    (
                        Err(DbPostSearchErr::Unauthorized(_)),
                        Err(DbPostSearchErr::Unauthorized(_)),
                        Ok(_),
                        Ok(_)
                    )
                ));
            }
        }
    }
}
