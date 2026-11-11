use crate::{Db, DbUserGet, XUuid, query::role_get_by_role_tx::DbRoleGetTxErr};
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

        let mut tx = pool
            .begin()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

        // get user
        let mut user = {
            let (q_my_user_join, q_followed_by_me_join, q_followed_by_me_field) =
                match user_username.is_empty() {
                    true => ("", "", "false as user_followed"),
                    false => (
                        "JOIN users my_user ON $2 = my_user.user_username",
                        "LEFT JOIN follow ON follower_user_id=my_user.user_id AND following_user_id=target_user.user_id",
                        "coalesce(follow_id > 0, false) as user_followed",
                    ),
                };

            let q_where_username = 'a: {
                if !username.is_empty() {
                    break 'a "target_user.user_username = $1";
                }
                ""
            };

            let q_where_email = 'a: {
                if !email.is_empty() {
                    break 'a "target_user.user_email = $1";
                }
                ""
            };

            let q_join_session = 'a: {
                if !session_token_is_empty {
                    break 'a "JOIN sessions ON target_user.user_id=session_user_id";
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
                    {q_followed_by_me_field},
                    target_user.user_id as user_id,
                    target_user.user_username as user_username,
                    target_user.user_email as user_email,
                    target_user.user_password as user_password,
                    target_user.user_support as user_support,
                    target_user.user_aboutme as user_aboutme,
                    target_user.user_state as user_state,
                    target_user.user_roles as user_roles,
                    array[]::varchar[] as user_roles_values,
                    target_user.user_pfp_image_hash as user_pfp_image_hash,
                    target_user.user_likes_count as user_likes_count,
                    target_user.user_followers_count as user_followers_count,
                    target_user.user_following_count as user_following_count,
                    coalesce(image_processed, false) as user_pfp_image_proccesed,
                    target_user.user_used_storage_bytes as user_used_storage_bytes,
                    target_user.user_max_storage_per_file_bytes as user_max_storage_per_file_bytes,
                    target_user.user_max_storage_bytes as user_max_storage_bytes,
                    target_user.user_max_storage_bytes as user_max_storage_bytes,
                    target_user.user_modified_at as user_modified_at,
                    target_user.user_created_at as user_created_at
                FROM users target_user
                LEFT JOIN files_images
                    ON target_user.user_pfp_image_hash = image_hash
                {q_my_user_join}
                {q_followed_by_me_join}
                {q_join_session}
                WHERE
                    {q_where_username}
                    {q_where_email}
                    {q_where_session}
                "
            );

            let query = AssertSqlSafe(query_str.as_str());
            let mut builder = sqlx::query_as(query);

            // TODO maybe use enum
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
            let result = builder.fetch_one(&mut *tx).await;

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
            let roles_values = self
                .role_get_by_role_ids_tx(&user.roles, &mut *tx)
                .await
                .map_err(|err| match err {
                    DbRoleGetTxErr::Db(err) => DbUserGetBuilderErr::Db(err),
                })?;

            user.roles_values = roles_values;
        }

        tx.commit()
            .await
            .inspect_err(|err| error!("follow_add {err}"))?;

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
        use catsquad_shared::SessionTokenKind;

        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.roles_values, vec![] as Vec<String>);

        db.role_add(0, SessionTokenKind::Root, "one", vec!["foo".to_string()])
            .await
            .unwrap();

        db.user_update_role_add(0, "hey2", "one").await.unwrap();

        let user = db
            .user_get_builder("", 0_u128.to_be_bytes(), "hey2", "")
            .await
            .unwrap();
        assert_eq!(user.roles_values, vec!["foo".to_string()]);

        db.role_add(0, SessionTokenKind::Root, "two", vec!["bar".to_string()])
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
