use crate::{Db, DbFileImage};
use catsquad_log::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum DbPostGetUnproccesedErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

impl Db {
    pub async fn post_get_unproccesed(&self) -> Result<Vec<DbFileImage>, DbPostGetUnproccesedErr> {
        let pool = &self.db;

        let query = "SELECT * FROM files_images WHERE image_processed = FALSE ORDER BY image_created_at ASC;";

        let result = sqlx::query_as(query).fetch_all(pool).await;

        debug!("query {query}\nresults {result:?}");

        let img = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostGetUnproccesedErr::Db(err));
            }
        };

        Ok(img)
        // let query = "SELECT *, user.* FROM post WHERE file.proccesed CONTAINS false ORDER BY created_at ASC;";

        // trace!("about to run {query}");

        // self.db
        //     .query(query)
        //     .await
        //     .check_good(|err| match err {
        //         err => {
        //             error!("unexpected db error {err}");
        //             DbPostGetUnproccesedErr::Db(err)
        //         }
        //     })
        //     .and_then_take_all(0)
    }
}

// test in /api/post_update_proccesed
