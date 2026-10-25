use crate::{Db, DbUserGet, XUuid};
use catsquad_log::prelude::*;
use catsquad_shared::Uuid;
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetBuilderErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,

    #[error("must provide only username or only session_token")]
    InvalidInput,
}

impl Db {
    /// provide only session_token or only username
    pub async fn user_get_builder(
        &self,
        user_username: impl Into<String>,
        session_token: Uuid,
        username: impl Into<String>,
        email: impl Into<String>,
    ) -> Result<DbUserGet, DbUserGetBuilderErr> {
        // TODO can combine get by username/email
        let pool = &self.db;
        let user_username = user_username.into();
        let username = username.into();
        let email = email.into();
        let session_token_is_empty = u128::from_be_bytes(session_token) == 0;

        // get user
        let mut user = {
            let q_join_followed_by_me = 'a: {
                if !user_username.is_empty() {
                    break 'a "LEFT JOIN follow ON follower_user_username=$2 AND following_user_username=user_username";
                }
                ""
            };

            let q_field_followed_by_me = 'a: {
                if !user_username.is_empty() {
                    break 'a "coalesce(follow_id > 0, false) as user_followed";
                }
                "false as user_followed"
            };

            let q_group_followed_by_me = 'a: {
                if !user_username.is_empty() {
                    break 'a ", follow_id";
                }
                ""
            };

            let q_where_username = 'a: {
                if !username.is_empty() {
                    break 'a "user_username = $1";
                }
                ""
            };

            let q_where_email = 'a: {
                if !email.is_empty() {
                    break 'a "user_email = $1";
                }
                ""
            };

            let q_join_session = 'a: {
                if !session_token_is_empty {
                    break 'a "JOIN sessions ON user_email=session_user_email";
                }
                ""
            };

            let q_where_session = 'a: {
                if !session_token_is_empty {
                    break 'a "session_token = $1";
                }
                ""
            };

            let query_str = format!(
                "SELECT
                {q_field_followed_by_me},
                user_username,
                user_email,
                user_password,
                user_support,
                user_aboutme,
                user_roles,
                array[]::varchar[] as user_roles_values,
                user_pfp_image_hash,
                user_likes_count,
                user_followers_count,
                user_following_count,
                coalesce(image_processed, false) as user_pfp_image_proccesed,
                user_used_storage_bytes,
                user_max_storage_per_file_bytes,
                user_max_storage_bytes,
                user_max_storage_bytes,
                user_modified_at,
                user_created_at
                    FROM users
                LEFT JOIN files_images
                    ON user_pfp_image_hash = image_hash
                {q_join_followed_by_me}
                {q_join_session}
                WHERE
                    {q_where_username}
                    {q_where_email}
                    {q_where_session}
                GROUP BY user_username, image_processed {q_group_followed_by_me}
                "
            );

            let query = AssertSqlSafe(query_str.as_str());
            let mut builder = sqlx::query_as(query);
            match (
                email.is_empty(),
                username.is_empty(),
                session_token_is_empty,
            ) {
                (true, false, true) => builder = builder.bind(username),
                (true, true, false) => builder = builder.bind(XUuid(session_token)),
                (false, true, true) => builder = builder.bind(email),
                (true, true, true) => {
                    return Err(DbUserGetBuilderErr::NotFound);
                }
                _ => {
                    error!(
                        "only email or only username or only session_token can by provided, current input: {} {} {:?}",
                        email, username, session_token
                    );

                    return Err(DbUserGetBuilderErr::InvalidInput);
                }
            }
            if !user_username.is_empty() {
                builder = builder.bind(user_username);
            }
            let result = builder.fetch_one(pool).await;

            debug!("{query_str}\n{result:#?}");

            let user: DbUserGet = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbUserGetBuilderErr::NotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserGetBuilderErr::Db(err));
                }
            };

            user
        };

        // get roles
        {
            let query = "SELECT
                                    role_id,
                                    role_values
                                FROM unnest($1)
                                INNER JOIN roles ON unnest=role_id
                                ";

            let result = sqlx::query_as(query)
                .bind(&user.roles)
                .fetch_all(pool)
                .await;

            debug!("{query}\n{result:#?}");

            let roles: Vec<(i64, Vec<String>)> = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserGetBuilderErr::Db(err));
                }
            };

            let mut roles_values = Vec::new();

            for role_id_a in &user.roles {
                let Some((_, values)) = roles
                    .iter()
                    .find(|(role_id_b, _)| *role_id_a == *role_id_b)
                    .cloned()
                else {
                    error!(
                        "role missing for user {}, expected role ids {:?}, received: {:#?}",
                        user.username, user.roles, roles
                    );
                    break;
                };

                roles_values.extend(values);
            }

            user.roles_values = roles_values;
        }

        Ok(user)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_get_builder() {
    init_log();

    let db = Db::test_db(0, "test_user_get_builder").await;
    let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
    let invite2 = db.invite_add(0, "hey2@hey.com", 10).await.unwrap();

    db.user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
        .await
        .unwrap();
    db.user_add(0, "hey2", "hey", invite2.token.clone(), 10, 10)
        .await
        .unwrap();

    // assert success
    {
        let user = db.user_get_by_username("", "hey2").await.unwrap();
        assert_eq!(user.username, "hey2");
    }

    // assert errors
    {
        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey3", "")
            .await;
        assert!(matches!(user, Err(DbUserGetBuilderErr::NotFound)));

        let user = db
            .user_get_builder("", 1_u128.to_be_bytes(), "hey3", "")
            .await;
        assert!(matches!(user, Err(DbUserGetBuilderErr::InvalidInput)));

        let user = db
            .user_get_builder("", 1_u128.to_be_bytes(), "", "hey3")
            .await;
        assert!(matches!(user, Err(DbUserGetBuilderErr::InvalidInput)));
    }

    // assert followed field
    {
        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.followed, false);

        let user = db
            .user_get_builder("hey", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.followed, false);

        db.follow_add(0, "hey", "hey2").await.unwrap();

        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.followed, false);

        let user = db
            .user_get_builder("hey", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.followed, true);

        let user = db
            .user_get_builder("hey2", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.followed, false);
    }

    // assert role field
    {
        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.roles_values, vec![] as Vec<String>);

        db.role_add(0, "one", vec!["foo".to_string()])
            .await
            .unwrap();

        db.user_update_role_add(0, "hey2", "one").await.unwrap();

        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.roles_values, vec!["foo".to_string()]);

        db.role_add(0, "two", vec!["bar".to_string()])
            .await
            .unwrap();

        db.user_update_role_add(0, "hey2", "two").await.unwrap();

        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.roles.len(), 2);
        assert!(user.roles_values.contains(&"foo".to_string()));
        assert!(user.roles_values.contains(&"bar".to_string()));
    }

    // assert get by sesson token
    {
        let session_token = db.session_add(0, "hey2@hey.com").await.unwrap();

        let user = db
            .user_get_builder("", session_token.token, "", "")
            .await
            .unwrap();

        assert_eq!(user.username, "hey2");
    }

    // assert get by email
    {
        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "", "hey2@hey.com")
            .await
            .unwrap();

        assert_eq!(user.username, "hey2");
    }

    // assert pfp field
    {
        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();

        assert_eq!(user.email, "hey2@hey.com");
        assert_eq!(user.pfp_image_hash, 0);
        assert_eq!(user.pfp_image_proccesed, false);

        db.user_update_pfp(0, "hey2", async move |_, _| {}, 10, 666, "webp", 10, 10)
            .await
            .unwrap();

        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.email, "hey2@hey.com");
        assert_eq!(user.pfp_image_hash, 666);
        assert_eq!(user.pfp_image_proccesed, false);

        db.file_image_update_proccsed(0, 666).await.unwrap();

        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.email, "hey2@hey.com");
        assert_eq!(user.pfp_image_hash, 666);
        assert_eq!(user.pfp_image_proccesed, true);
    }
}
