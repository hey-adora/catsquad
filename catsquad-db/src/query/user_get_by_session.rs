use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetBySessionErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

// impl Db {
//     pub async fn user_get_by_session(
//         &self,
//         user_username: impl Into<String>,
//         username: impl Into<String>,
//     ) -> Result<DbUserGet, DbUserGetByUsernameErr> {
//         let pool = &self.db;
//         let user_username = user_username.into();
//         let username = username.into();

//         // TODO can combine get by username/email
//         let q_followed_by_me = 'a: {
//             if !user_username.is_empty() {
//                 break 'a "LEFT JOIN follow ON follower_user_username=$2 AND following_user_username=user_username";
//             }
//             ""
//         };

//         let q_followed_by_me_field = 'a: {
//             if !user_username.is_empty() {
//                 break 'a "coalesce(follow_id > 0, false) as user_followed";
//             }
//             "false as user_followed"
//         };

//         let query_str = format!(
//             "SELECT
//                 {q_followed_by_me_field},
//                 user_username,
//                 user_email,
//                 user_password,
//                 user_support,
//                 user_aboutme,
//                 user_pfp_image_hash,
//                 user_likes_count,
//                 user_followers_count,
//                 user_following_count,
//                 coalesce(image_processed, false) as user_pfp_image_proccesed,
//                 user_used_storage_bytes,
//                 user_max_storage_per_file_bytes,
//                 user_max_storage_bytes,
//                 user_max_storage_bytes,
//                 user_modified_at,
//                 user_created_at
//                     FROM users
//                 LEFT JOIN files_images
//                     ON user_pfp_image_hash = image_hash
//                 {q_followed_by_me}
//                 WHERE
//                     user_username = $1"
//         );

//         let query = AssertSqlSafe(query_str.as_str());
//         let mut builder = sqlx::query_as(query).bind(username);
//         if !user_username.is_empty() {
//             builder = builder.bind(user_username);
//         }
//         let result = builder.fetch_one(pool).await;

//         debug!("{query_str}\n{result:#?}");

//         let users = match result {
//             Ok(v) => v,
//             Err(sqlx::Error::RowNotFound) => {
//                 return Err(DbUserGetByUsernameErr::NotFound);
//             }
//             Err(err) => {
//                 error!("unexpected db error {err}");
//                 return Err(DbUserGetByUsernameErr::Db(err));
//             }
//         };

//         debug!("query {query_str}\nresults {users:?}");

//         Ok(users)
//     }
// }

// #[cfg(test)]
// #[tokio::test]
// async fn test_user_get_by_username() {
//     init_log();

//     let db = Db::test_db(0, "test_user_get_by_username").await;
//     let invite = db.invite_add(0, "hey@hey.com", 10).await.unwrap();
//     let invite2 = db.invite_add(0, "hey2@hey.com", 10).await.unwrap();

//     db.user_add(0, "hey", "hey", invite.token.clone(), 10, 10)
//         .await
//         .unwrap();
//     db.user_add(0, "hey2", "hey", invite2.token.clone(), 10, 10)
//         .await
//         .unwrap();
//     // assert success
//     {
//         let user = db.user_get_by_username("", "hey2").await.unwrap();
//         assert_eq!(user.username, "hey2");
//     }

//     // assert errors
//     {
//         let user = db.user_get_by_username("", "hey3").await;
//         assert!(matches!(user, Err(DbUserGetByUsernameErr::NotFound)));
//     }

//     // assert followed field
//     {
//         let user = db.user_get_by_username("", "hey2").await.unwrap();
//         assert_eq!(user.followed, false);
//         let user = db.user_get_by_username("hey", "hey2").await.unwrap();
//         assert_eq!(user.followed, false);
//         db.follow_add(0, "hey", "hey2").await.unwrap();
//         let user = db.user_get_by_username("", "hey2").await.unwrap();
//         assert_eq!(user.followed, false);
//         let user = db.user_get_by_username("hey", "hey2").await.unwrap();
//         assert_eq!(user.followed, true);
//         let user = db.user_get_by_username("hey2", "hey2").await.unwrap();
//         assert_eq!(user.followed, false);
//     }
// }
