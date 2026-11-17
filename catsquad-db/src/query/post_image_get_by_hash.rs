use crate::{
    Db, DbFileImage, DbUserGetBySessionAndRolesErr, DbUserGetBySessionAndRolesSimpleErr,
    auth_perms, auth_username,
};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PostState, ROLE_VALUE_PERMISSION_SEE_HIDDEN, SessionTokenKind, UserState,
    contains_perms_or_is_root,
};
use sqlx::{FromRow, Row};

#[derive(Debug, thiserror::Error)]
pub enum DbPostImageGetByHashErr {
    #[error("post not found")]
    NotFound,

    #[error("unauthorized {0}")]
    Unauthorized(String),

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal server error")]
    InternalServerError,
}

fn from_user_get_by_session_and_roles_err(
    value: DbUserGetBySessionAndRolesErr,
) -> DbPostImageGetByHashErr {
    match DbUserGetBySessionAndRolesSimpleErr::from(value) {
        DbUserGetBySessionAndRolesSimpleErr::Unauthorized(err) => {
            DbPostImageGetByHashErr::Unauthorized(err)
        }
        DbUserGetBySessionAndRolesSimpleErr::Db(err) => DbPostImageGetByHashErr::Db(err),
    }
}

impl Db {
    pub async fn post_image_get_by_hash(
        &self,
        auth_session_token: SessionTokenKind,
        post_id: i64,
        image_hash: i64,
    ) -> Result<DbFileImage, DbPostImageGetByHashErr> {
        let pool = &self.db;

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("post_image_get_by_hash {err}"))?;

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

        // nested array because its difficult to relate post and images without unnest'ing hashes
        // TODO maybe try sql WITH again

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
                                image_created_at,
                                post_user_username,
                                post_user_state,
                                post_state
                            FROM (
                                    SELECT
                                        unnest(post_images_hashes) as post_image_hash,
                                        post_state,
                                        user_username as post_user_username,
                                        user_state as post_user_state
                                    FROM posts
                                    JOIN users ON post_user_id = user_id
                                    WHERE post_id = $1
                                )
                            INNER JOIN files_images
                                ON post_image_hash=image_hash
                            WHERE image_hash = $2
                            LIMIT 1";

        // debug!("about to run {query}");

        let result = sqlx::query(query)
            .bind(post_id)
            .bind(image_hash)
            .fetch_one(&mut *tx)
            .await;

        debug!("query {query}\nresults {result:?}");

        let row = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbPostImageGetByHashErr::NotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostImageGetByHashErr::Db(err));
            }
        };

        // auth
        {
            let post_state: String = row.get("post_state");
            let post_user_username: String = row.get("post_user_username");
            let post_user_state: String = row.get("post_user_state");

            let post_state = PostState::try_from(post_state)
                .inspect_err(|err| error!("{err}"))
                .map_err(|_| DbPostImageGetByHashErr::InternalServerError)?;

            let post_user_state = UserState::try_from(post_user_state)
                .inspect_err(|err| error!("{err}"))
                .map_err(|_| DbPostImageGetByHashErr::InternalServerError)?;

            let post_is_mine = post_user_username == user_username;
            let post_is_active = post_state.is_active();
            let post_user_state = post_user_state.is_active();

            let show_image_bytes =
                (post_is_active && post_user_state) || (post_is_mine || can_see_hidden);

            if !show_image_bytes {
                return Err(DbPostImageGetByHashErr::Unauthorized(
                    "unauthorized".to_string(),
                ));
            }
        }

        let image = DbFileImage::from_row(&row)
            .inspect_err(|err| error!("post_image_get_by_hash from_row {}", err.to_string()))?;

        tx.commit()
            .await
            .inspect_err(|err| error!("post_image_get_by_hash {err}"))?;

        Ok(image)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_image_get_by_hash() {
    // TODO finish the error testing stuff

    use catsquad_shared::SessionTokenKind;

    use crate::{DbPostUpdateImageAddErr, user_add_with_roles};
    init_log();

    let db = Db::test_db(0, "test_post_image_get_by_hash").await;

    let (user1, user1_token) = user_add_with_roles(
        &db,
        0,
        100,
        100,
        "hey",
        "pss",
        "hey@heyadora.com",
        Vec::<String>::new(),
    )
    .await;

    let (user2, user2_token) = user_add_with_roles(
        &db,
        0,
        100,
        100,
        "hey2",
        "pss",
        "hey2@heyadora.com",
        Vec::<String>::new(),
    )
    .await;

    let post1 = db
        .post_add(0, user1.username.clone(), "title1", "description1", "tags")
        .await
        .unwrap();

    let image = db
        .post_update_image_add(0, user1.username.clone(), post1.id, 19, 666, "ico", 10, 15)
        .await
        .unwrap();

    let _image2 = db
        .post_update_image_add(0, user1.username.clone(), post1.id, 29, 266, "png", 20, 25)
        .await
        .unwrap();

    let success_fn =
        move |result: Result<DbFileImage, DbPostImageGetByHashErr>| matches!(result, Ok(_));
    let unauthorized_fn = move |result: Result<DbFileImage, DbPostImageGetByHashErr>| {
        matches!(result, Err(DbPostImageGetByHashErr::Unauthorized(_)))
    };

    let img_get_fn =
        async |token: SessionTokenKind| db.post_image_get_by_hash(token, post1.id, 666).await;

    let test_get_fn = async || {
        let result1 = img_get_fn(user1_token).await;
        let result2 = img_get_fn(user2_token).await;
        let result3 = img_get_fn(SessionTokenKind::None).await;
        let result4 = img_get_fn(SessionTokenKind::Root).await;
        (result1, result2, result3, result4)
    };

    // assert not found
    {
        let result = db.post_image_get_by_hash(user1_token, post1.id, 667).await;

        assert!(matches!(result, Err(DbPostImageGetByHashErr::NotFound)));
    }

    // assert - post draft - user active
    {
        let (result1, result2, result3, result4) = test_get_fn().await;

        assert!(success_fn(result1));
        assert!(unauthorized_fn(result2));
        assert!(unauthorized_fn(result3));
        assert!(success_fn(result4));
    }

    db.post_update_state(0, user1.username.clone(), post1.id, PostState::Active)
        .await
        .unwrap();

    // assert - post active - user active
    {
        let (result1, result2, result3, result4) = test_get_fn().await;

        assert!(success_fn(result1));
        assert!(success_fn(result2));
        assert!(success_fn(result3));
        assert!(success_fn(result4));
    }

    db.post_update_state(0, user1.username.clone(), post1.id, PostState::Hidden)
        .await
        .unwrap();

    // assert - post hidden - user active
    {
        let (result1, result2, result3, result4) = test_get_fn().await;

        assert!(success_fn(result1));
        assert!(unauthorized_fn(result2));
        assert!(unauthorized_fn(result3));
        assert!(success_fn(result4));
    }

    db.post_update_state(0, user1.username.clone(), post1.id, PostState::Active)
        .await
        .unwrap();

    db.user_update_state(
        async |_| String::new(),
        0,
        SessionTokenKind::Root,
        vec![user1.username.clone()],
        UserState::Suspended,
    )
    .await
    .unwrap();

    // assert - post active - user suspended
    {
        let (result1, result2, result3, result4) = test_get_fn().await;

        assert!(unauthorized_fn(result1));
        assert!(unauthorized_fn(result2));
        assert!(unauthorized_fn(result3));
        assert!(success_fn(result4));
    }

    db.post_update_state(0, user1.username.clone(), post1.id, PostState::Hidden)
        .await
        .unwrap();

    // assert - post hidden - user suspended
    {
        let (result1, result2, result3, result4) = test_get_fn().await;

        assert!(unauthorized_fn(result1));
        assert!(unauthorized_fn(result2));
        assert!(unauthorized_fn(result3));
        assert!(success_fn(result4));
    }

    db.role_update_value(0, user2.roles[0], vec![ROLE_VALUE_PERMISSION_SEE_HIDDEN])
        .await
        .unwrap();

    // assert - see hidden
    {
        let (result1, result2, result3, result4) = test_get_fn().await;

        assert!(unauthorized_fn(result1));
        assert!(success_fn(result2));
        assert!(unauthorized_fn(result3));
        assert!(success_fn(result4));
    }
}
