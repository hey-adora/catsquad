use catsquad_log::prelude::*;

use crate::{Db, DbUser};

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn user_get_all(&self) -> Result<Vec<DbUser>, DbUserGetAllErr> {
        let pool = &self.db;
        let query = "SELECT
                            user_id,
                            user_username,
                            user_email,
                            user_password,
                            user_support,
                            user_aboutme,
                            user_state,
                            user_roles,
                            user_pfp_image_hash,
                            user_likes_count,
                            user_followers_count,
                            user_following_count,
                            coalesce(image_processed, false) as user_pfp_image_proccesed,
                            user_used_storage_bytes,
                            user_max_storage_per_file_bytes,
                            user_max_storage_bytes,
                            user_max_storage_bytes,
                            user_state_modified_at,
                            user_ip,
                            user_agent,
                            user_modified_at,
                            user_created_at
                                FROM users
                            LEFT JOIN files_images
                                ON user_pfp_image_hash = image_hash
                            ORDER BY user_created_at DESC
                            ";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let users = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbUserGetAllErr::Db(err));
            }
        };

        Ok(users)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_get_all() {
    use crate::user_add_for_testing;

    init_log();

    let db = Db::test_db(0, "test_user_get_all").await;

    let user = user_add_for_testing(&db, 0, "hey", "pss", "hey@heyadora.com", 10, 10).await;

    let users = db.user_get_all().await.unwrap();
    assert_eq!(users.len(), 1);
}
