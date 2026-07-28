use catsquad_log::prelude::*;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

use crate::{Db, DbInvite, DbPost, SurrealCheckUtils, SurrealErrUtils, SurrealSerializeUtils};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbPostGetUnproccesedErr {
    #[error("DB error {0}")]
    Db(#[from] surrealdb::Error),
}

impl Db {
    pub async fn post_get_unproccesed(&self) -> Result<Vec<DbPost>, DbPostGetUnproccesedErr> {
        let query = "SELECT *, user.* FROM post WHERE file.proccesed CONTAINS false ORDER BY created_at ASC;";

        trace!("about to run {query}");

        self.db
            .query(query)
            .await
            .check_good(|err| match err {
                err => {
                    error!("unexpected db error {err}");
                    DbPostGetUnproccesedErr::Db(err)
                }
            })
            .and_then_take_all(0)
    }
}

// test in /api/post_update_proccesed
