use crate::{Db, DbPost};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPostGetAllErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_get_all(&self) -> Result<Vec<DbPost>, DbPostGetAllErr> {
        let pool = &self.db;
        let query = "SELECT * FROM posts ORDER BY post_created_at DESC";

        trace!("about to run {query}");

        let result = sqlx::query_as(query).fetch_all(pool).await;

        let posts = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostGetAllErr::Db(err));
            }
        };

        Ok(posts)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_get_all() {
    init_log();

    let db = Db::test_db(0, "test_post_get_all").await;

    let invite1 = db.invite_add(0, "hey@heyadora.com", 1).await.unwrap();

    let user = db
        .user_add(0, "hey", "hey", invite1.token.clone(), 10, 10)
        .await
        .unwrap();

    let post1 = db
        .post_add(0, user.username.clone(), "title1", "description1", "tags")
        .await
        .unwrap();

    let _file = db
        .post_update_file_add(0, user.username.clone(), post1.id, 10, 333, "png", 10, 10)
        .await
        .unwrap();

    let posts = db.post_get_all().await.unwrap();
    assert_eq!(posts.len(), 1);
}
