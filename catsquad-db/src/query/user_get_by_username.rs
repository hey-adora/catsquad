use std::ops::Add;

use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetByUsernameErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbUserGet {
    #[sqlx(rename = "user_username")]
    pub username: String,
    #[sqlx(rename = "user_email")]
    pub email: String,
    #[sqlx(rename = "user_password")]
    pub password: String,
    #[sqlx(rename = "user_support")]
    pub support: String,
    #[sqlx(rename = "user_aboutme")]
    pub aboutme: String,
    #[sqlx(rename = "user_roles")]
    pub roles: Vec<i64>,
    #[sqlx(rename = "user_roles_values")]
    pub roles_values: Vec<String>,
    #[sqlx(rename = "user_pfp_image_hash")]
    pub pfp_image_hash: i64,
    #[sqlx(default, rename = "user_pfp_image_proccesed")]
    pub pfp_image_proccesed: bool,
    #[sqlx(rename = "user_likes_count")]
    #[sqlx(try_from = "i64")]
    pub likes_count: u32,
    #[sqlx(default, rename = "user_followed")]
    pub followed: bool,
    #[sqlx(rename = "user_followers_count")]
    #[sqlx(try_from = "i64")]
    pub followers_count: u32,
    #[sqlx(rename = "user_following_count")]
    #[sqlx(try_from = "i64")]
    pub following_count: u32,
    #[sqlx(rename = "user_used_storage_bytes")]
    #[sqlx(try_from = "i64")]
    pub used_storage_bytes: u32,
    #[sqlx(rename = "user_max_storage_per_file_bytes")]
    #[sqlx(try_from = "i64")]
    pub max_storage_per_image_bytes: u32,
    #[sqlx(rename = "user_max_storage_bytes")]
    #[sqlx(try_from = "i64")]
    pub max_storage_bytes: u32,
    #[sqlx(rename = "user_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "user_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

impl Db {
    pub async fn user_get_by_username(
        &self,
        user_username: impl Into<String>,
        username: impl Into<String>,
    ) -> Result<DbUserGet, DbUserGetByUsernameErr> {
        // TODO can combine get by username/email
        let pool = &self.db;
        let user_username = user_username.into();
        let username = username.into();

        // let images_sorted = images_hashes
        //     .clone()
        //     .into_iter()
        //     .map(|hash| images.iter().find(|img| img.hash == hash).cloned())
        //     .collect::<Option<Vec<DbFileImage>>>()
        //     .ok_or(DbPostGetByKeyErr::PostNotFound)?;

        // JOIN unnest(user_roles) r1
        // LEFT JOIN roles
        //     ON r1=role_id
        // LEFT JOIN unnest(role_values) a1
        //     ON true
        // get user
        // array_remove(array_agg(a1), NULL) as user_roles,
        let mut user = {
            let q_followed_by_me = 'a: {
                if !user_username.is_empty() {
                    break 'a "LEFT JOIN follow ON follower_user_username=$2 AND following_user_username=user_username";
                }
                ""
            };

            let q_followed_by_me_field = 'a: {
                if !user_username.is_empty() {
                    break 'a "coalesce(follow_id > 0, false) as user_followed";
                }
                "false as user_followed"
            };

            let q_followed_by_me_group = 'a: {
                if !user_username.is_empty() {
                    break 'a ", follow_id";
                }
                ""
            };
            let query_str = format!(
                "SELECT
                {q_followed_by_me_field},
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
                {q_followed_by_me}
                WHERE
                    user_username = $1
                GROUP BY user_username, image_processed {q_followed_by_me_group}
                "
            );

            let query = AssertSqlSafe(query_str.as_str());
            let mut builder = sqlx::query_as(query).bind(username);
            if !user_username.is_empty() {
                builder = builder.bind(user_username);
            }
            let result = builder.fetch_one(pool).await;

            debug!("{query_str}\n{result:#?}");

            let user: DbUserGet = match result {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(DbUserGetByUsernameErr::NotFound);
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserGetByUsernameErr::Db(err));
                }
            };

            user
        };

        // get roles
        {
            // role_id int8 PRIMARY KEY generated always as identity,
            // role_name varchar NOT NULL,
            // role_values varchar[] DEFAULT array[]::varchar[],
            // role_modified_at timestamp NOT NULL,
            // role_created_at timestamp NOT NULL
            let query = "SELECT
                                    role_id,
                                    role_values
                                FROM unnest($1)
                                INNER JOIN roles ON unnest=role_id
                                ";
            // (SELECT unnest(post_images_hashes) as post_image_hash
            //     FROM posts
            //     WHERE post_id = $1)

            let result = sqlx::query_as(query)
                .bind(&user.roles)
                .fetch_all(pool)
                .await;

            debug!("{query}\n{result:#?}");

            let roles: Vec<(i64, Vec<String>)> = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbUserGetByUsernameErr::Db(err));
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

        // let q_followed_by_me_group_field = 'a: {
        //     if !user_username.is_empty() {
        //         break 'a ", follow_id";
        //     }
        //     ""
        // };

        // 0.div;

        // array_agg(a1) as user_roles,
        // LEFT JOIN roles
        //     ON user_username=role_user_username
        // LEFT JOIN unnest(coalesce(role_values, array[]::varchar[])) a1
        //     ON true
        // coalesce(array_agg(a1), array[]::varchar[]) as user_roles,
        // array[]::varchar[] as user_roles,
        // LEFT JOIN unnest(coalesce(role_values, array[]::varchar[])) a1
        // coalesce(array_agg(a1), array[]::varchar[]) as user_roles,
        // array[]::varchar[] as user_roles,

        Ok(user)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_get_by_username() {
    init_log();

    let db = Db::test_db(0, "test_user_get_by_username").await;
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
        let user = db.user_get_by_username("", "hey3").await;
        assert!(matches!(user, Err(DbUserGetByUsernameErr::NotFound)));
    }

    // assert followed field
    {
        let user = db.user_get_by_username("", "hey2").await.unwrap();
        assert_eq!(user.followed, false);
        let user = db.user_get_by_username("hey", "hey2").await.unwrap();
        assert_eq!(user.followed, false);
        db.follow_add(0, "hey", "hey2").await.unwrap();
        let user = db.user_get_by_username("", "hey2").await.unwrap();
        assert_eq!(user.followed, false);
        let user = db.user_get_by_username("hey", "hey2").await.unwrap();
        assert_eq!(user.followed, true);
        let user = db.user_get_by_username("hey2", "hey2").await.unwrap();
        assert_eq!(user.followed, false);
    }

    // assert role field
    {
        // let user = db.user_get_by_username("", "hey2").await.unwrap();
        // assert_eq!(user.roles, vec![] as Vec<String>);

        // db.role_add(0, "one", vec!["foo".to_string()])
        //     .await
        //     .unwrap();

        // let user = db.user_get_by_username("", "hey2").await.unwrap();
        // assert_eq!(user.roles, vec!["foo".to_string()]);

        // db.role_add(0, "two", vec!["bar".to_string()])
        //     .await
        //     .unwrap();

        // let user = db.user_get_by_username("", "hey2").await.unwrap();
        // assert_eq!(user.roles.len(), 2);
        // assert!(user.roles.contains(&"foo".to_string()));
        // assert!(user.roles.contains(&"bar".to_string()));
    }
}
