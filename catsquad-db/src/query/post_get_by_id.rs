use crate::{
    Db, DbFileImage, XTimestamp,
    query::user_get_by_session_and_roles::{
        DbUserGetBySessionAndRolesErr, DbUserGetBySessionAndRolesSimpleErr, auth_perms,
        auth_username,
    },
};
use catsquad_log::prelude::*;
use catsquad_shared::{
    POST_STATE_HIDDEN, PostState, ROLE_VALUE_PERMISSION_SEE_HIDDEN, SessionTokenKind, UserState,
    contains_perms_or_is_root,
};
use sqlx::{AssertSqlSafe, FromRow};

#[derive(Debug, thiserror::Error)]
pub enum DbPostGetByKeyErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal server error")]
    InternalServerErr,
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbPostGet {
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
    #[sqlx(rename = "post_user_followers")]
    #[sqlx(try_from = "i64")]
    pub followers: u32,
    #[sqlx(rename = "post_user_followed")]
    pub followed: bool,
    #[sqlx(rename = "post_liked")]
    pub liked: bool,
    #[sqlx(rename = "post_likes_count")]
    #[sqlx(try_from = "i64")]
    pub likes_count: u32,
    #[sqlx(rename = "post_size_bytes")]
    #[sqlx(try_from = "i64")]
    pub size_bytes: u32,
    #[sqlx(skip)]
    pub images: Vec<DbFileImage>,
    #[sqlx(rename = "post_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "post_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

fn from_user_get_by_session_and_roles_err(
    value: DbUserGetBySessionAndRolesErr,
) -> DbPostGetByKeyErr {
    match DbUserGetBySessionAndRolesSimpleErr::from(value) {
        DbUserGetBySessionAndRolesSimpleErr::Unauthorized(err) => {
            DbPostGetByKeyErr::Unauthorized(err)
        }
        DbUserGetBySessionAndRolesSimpleErr::Db(err) => DbPostGetByKeyErr::Db(err),
    }
}

impl Db {
    pub async fn post_get_by_id(
        &self,
        auth_session_token: SessionTokenKind,
        post_id: i64,
    ) -> Result<DbPostGet, DbPostGetByKeyErr> {
        let pool = &self.db;

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("comment_search {err}"))?;

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

        let (
            q_join_my_user,
            //
            q_liked_by_me_join,
            q_liked_by_me_field,
            //
            q_followed_by_me_join,
            q_followed_by_me_field,
        ) = match user_username.is_empty() {
            true => (
                "",
                //
                "",
                "false as post_liked",
                //
                "",
                "false as post_user_followed",
            ),
            false => (
                "JOIN users my_user ON $2=my_user.user_username",
                //
                "LEFT JOIN posts_likes ON post_id=post_like_post_id AND post_like_user_id = my_user.user_id",
                "coalesce(post_like_post_id > 0, false) as post_liked",
                //
                "LEFT JOIN follow ON my_user.user_id=follower_user_id AND post_user.user_id=following_user_id",
                "coalesce(follow_id > 0, false) as post_user_followed",
            ),
        };

        // get post
        let (images_hashes, mut post) = {
            let query_str = format!(
                "
                SELECT
                        post_images_hashes,
                        post_user.user_state,

                        post_user.user_pfp_image_hash as post_user_pfp_hash,
                        coalesce(image_processed, false) as post_user_pfp_proccesed,

                        {q_liked_by_me_field},

                        post_user.user_followers_count as post_user_followers,
                        {q_followed_by_me_field},

                        post_id,
                        post_user_id,
                        post_user.user_username as post_user_username,
                        post_user.user_state as post_user_state,
                        post_state,
                        post_title,
                        post_description,
                        post_tags,
                        post_likes_count,
                        post_size_bytes,
                        post_modified_at,
                        post_created_at
                    FROM posts

                    JOIN users post_user ON post_user_id=post_user.user_id
                    LEFT JOIN files_images ON user_pfp_image_hash=image_hash

                    {q_join_my_user}
                    {q_liked_by_me_join}
                    {q_followed_by_me_join}

                    WHERE post_id = $1 AND post_state != 'draft'
                "
            );
            // AND post_user.user_state = ANY($3)
            let query = AssertSqlSafe(query_str.as_str());
            let result = sqlx::query(query)
                .bind(post_id)
                .bind(user_username.clone())
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query_str}\nresults {result:#?}");

            let pg_row = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbPostGetByKeyErr::PostNotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostGetByKeyErr::Db(err));
                }
            };

            let post = DbPostGet::from_row(&pg_row);
            let post = match post {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostGetByKeyErr::Db(err));
                }
            };

            let images_hashes = <(Vec<i64>, String)>::from_row(&pg_row);
            let (images_hashes, post_user_state) = match images_hashes {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostGetByKeyErr::Db(err));
                }
            };

            // auth
            {
                let post_state = PostState::try_from(post.state.as_str())
                    .inspect_err(|err| error!("{err}"))
                    .map_err(|_err| DbPostGetByKeyErr::InternalServerErr)?;

                let post_user_state = UserState::try_from(post_user_state)
                    .inspect_err(|err| {
                        error!("invalid user_state {err} for {}", post.user_username)
                    })
                    .map_err(|_| DbPostGetByKeyErr::PostNotFound)?;

                let post_is_mine = post.user_username == user_username;
                let post_user_is_active = post_user_state.is_active();
                let post_is_active = post_state.is_active();

                let can_get_post =
                    (post_is_active && post_user_is_active) || (post_is_mine || can_see_hidden);

                if !can_get_post {
                    return Err(DbPostGetByKeyErr::Unauthorized(
                        "post is hidden".to_string(),
                    ));
                }
            }

            (images_hashes, post)
        };

        // get images
        {
            let query = "SELECT
                                    image_hash,
                                    image_extension,
                                    image_kind,
                                    image_size_bytes,
                                    image_processed,
                                    image_used_count,
                                    image_width,
                                    image_height,
                                    image_modified_at,
                                    image_created_at
                                 FROM unnest($2)
                                  INNER JOIN files_images ON unnest=image_hash
                                 ";
            // (SELECT unnest(post_images_hashes) as post_image_hash
            //     FROM posts
            //     WHERE post_id = $1)

            let result = sqlx::query_as(query)
                .bind(post_id) // TODO i think this is useless
                .bind(&images_hashes)
                .fetch_all(&mut *tx)
                .await;

            let images: Vec<DbFileImage> = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbPostGetByKeyErr::Db(err));
                }
            };

            let images_sorted = images_hashes
                .clone()
                .into_iter()
                .map(|hash| images.iter().find(|img| img.hash == hash).cloned())
                .collect::<Option<Vec<DbFileImage>>>()
                .ok_or(DbPostGetByKeyErr::PostNotFound)?;

            post.images = images_sorted;
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("comment_search {err}"))?;

        Ok(post)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_get_by_id() {
    use crate::user_add_with_roles;
    use catsquad_shared::PostState;
    use catsquad_shared::UserState;

    init_log();

    let db = Db::test_db(0, "test_post_get_by_id").await;

    let (user1, mut user1_token) = user_add_with_roles(
        &db,
        0,
        10,
        10,
        "hey",
        "pss",
        "hey@heyadora.com",
        vec![] as Vec<&'static str>,
    )
    .await;

    let (user2, user2_token) = user_add_with_roles(
        &db,
        1,
        10,
        10,
        "hey2",
        "pss",
        "hey2@heyadora.com",
        vec![] as Vec<&'static str>,
    )
    .await;

    // create post
    let post1 = {
        let post1 = db
            .post_add(0, user1.username.clone(), "title1", "description1", "tags")
            .await
            .unwrap();

        db.post_update_image_add(0, user1.username.clone(), post1.id, 10, 555, "jpg", 10, 15)
            .await
            .unwrap();

        post1
    };

    let user_state_fn = async |username: &str, state: UserState| {
        db.user_update_state(
            async |_| String::new(),
            0,
            SessionTokenKind::Root,
            vec![username.to_string()],
            state,
        )
        .await
        .unwrap();

        let user_email = match username {
            "hey" => "hey@heyadora.com",
            "hey2" => "hey2@heyadora.com",
            _ => unreachable!(),
        };

        if state == UserState::Active {
            let session = db.session_add(0, user_email).await.unwrap();
            SessionTokenKind::User(session.token)
        } else {
            SessionTokenKind::None
        }
    };

    let post_state_fn = async |username: &str, state: PostState| {
        db.post_update_state(0, username.to_string(), post1.id.clone(), state)
            .await
            .unwrap();
    };

    let user_see_hidden_fn = async |username: &str, see_hidden: bool| {
        let user_role_id = match username {
            "hey" => user1.roles[0],
            "hey2" => user2.roles[0],
            _ => unreachable!(),
        };
        let permissions = if see_hidden {
            vec![ROLE_VALUE_PERMISSION_SEE_HIDDEN]
        } else {
            vec![]
        };
        db.role_update_value(SessionTokenKind::Root, 0, user_role_id, permissions)
            .await
            .unwrap();
    };

    // asssert error not found
    {
        let result = db.post_get_by_id(user1_token, post1.id.clone()).await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::PostNotFound)));

        let result = db.post_get_by_id(user2_token, post1.id.clone()).await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::PostNotFound)));
    }

    // assert - post active - user active
    {
        user1_token = user_state_fn("hey", UserState::Active).await;
        post_state_fn("hey", PostState::Active).await;

        let result = db.post_get_by_id(user1_token, post1.id.clone()).await;
        assert!(matches!(result, Ok(_)));

        let result = db.post_get_by_id(user2_token, post1.id.clone()).await;
        assert!(matches!(result, Ok(_)));

        let result = db
            .post_get_by_id(SessionTokenKind::None, post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));

        let result = db
            .post_get_by_id(SessionTokenKind::Root, post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));
    }

    // assert - post hidden - user active
    {
        user1_token = user_state_fn("hey", UserState::Active).await;
        post_state_fn("hey", PostState::Hidden).await;

        let result = db.post_get_by_id(user1_token, post1.id.clone()).await;
        assert!(matches!(result, Ok(_)));

        let result = db.post_get_by_id(user2_token, post1.id.clone()).await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized(_))));

        let result = db
            .post_get_by_id(SessionTokenKind::None, post1.id.clone())
            .await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized(_))));

        let result = db
            .post_get_by_id(SessionTokenKind::Root, post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));
    }

    // assert - post hidden - user hidden
    {
        user_state_fn("hey", UserState::Suspended).await;
        post_state_fn("hey", PostState::Hidden).await;

        let result = db.post_get_by_id(user1_token, post1.id.clone()).await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized(_))));

        let result = db.post_get_by_id(user2_token, post1.id.clone()).await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized(_))));

        let result = db
            .post_get_by_id(SessionTokenKind::None, post1.id.clone())
            .await;
        assert!(matches!(result, Err(DbPostGetByKeyErr::Unauthorized(_))));

        let result = db
            .post_get_by_id(SessionTokenKind::Root, post1.id.clone())
            .await;
        assert!(matches!(result, Ok(_)));
    }

    // assert - post hidden - user hidden - see_hidden_perms
    {
        user_see_hidden_fn("hey2", true).await;

        let result = db.post_get_by_id(user2_token, post1.id.clone()).await;
        assert!(matches!(result, Ok(_)));

        user_see_hidden_fn("hey2", false).await;
    }

    user1_token = user_state_fn("hey", UserState::Active).await;
    post_state_fn("hey", PostState::Active).await;

    // assert pfp state of post author
    {
        db.post_update_state(
            0,
            user1.username.clone(),
            post1.id.clone(),
            PostState::Active,
        )
        .await
        .unwrap();

        assert_eq!(post1.user_pfp_hash, 0);
        assert_eq!(post1.user_pfp_proccesed, false);

        db.user_update_pfp(
            0,
            user1.username.clone(),
            async move |_, _| (),
            10,
            666,
            "webp",
            10,
            10,
        )
        .await
        .unwrap();

        let post1 = db
            .post_get_by_id(user2_token, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.user_pfp_hash, 666);
        assert_eq!(post1.user_pfp_proccesed, false);

        db.file_image_update_proccsed(0, 666).await.unwrap();

        let post1 = db
            .post_get_by_id(user2_token, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.user_pfp_hash, 666);
        assert_eq!(post1.user_pfp_proccesed, true);
    }

    // assert post like state
    {
        let post1 = db
            .post_get_by_id(user2_token, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.liked, false);

        db.post_like_add(0, user2.username.clone(), post1.id)
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user2_token, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.liked, true);

        let post1 = db
            .post_get_by_id(user1_token, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.liked, false);

        let post1 = db
            .post_get_by_id(SessionTokenKind::None, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.liked, false);
    }

    // assert follow state
    {
        let post1 = db
            .post_get_by_id(user2_token, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.followed, false);

        db.follow_add(0, user2.username.clone(), user1.username.clone())
            .await
            .unwrap();

        let post1 = db
            .post_get_by_id(user2_token, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.followers, 1);
        assert_eq!(post1.followed, true);

        let post1 = db
            .post_get_by_id(user1_token, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.followed, false);

        let post1 = db
            .post_get_by_id(SessionTokenKind::None, post1.id.clone())
            .await
            .unwrap();

        assert_eq!(post1.followed, false);
    }
}
