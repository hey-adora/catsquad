use crate::{Db, DbPost, XTimestamp};
use catsquad_log::prelude::*;
use sqlx::AssertSqlSafe;

#[derive(Debug, thiserror::Error)]
pub enum DbPostUpdateBuilderTextErr {
    #[error("post not found")]
    PostNotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),
}

#[derive(Clone, Copy)]
pub enum DbPostUpdateBuilderTextField {
    Title,
    Description,
    Tags,
}

impl DbPostUpdateBuilderTextField {
    pub fn as_str(&self) -> &'static str {
        match self {
            DbPostUpdateBuilderTextField::Title => "post_title",
            DbPostUpdateBuilderTextField::Description => "post_description",
            DbPostUpdateBuilderTextField::Tags => "post_tags",
        }
    }
}

impl Db {
    pub async fn post_update_builder_text(
        &self,
        time: u64,
        user_username: impl Into<String>,
        post_id: i64,
        update_field: DbPostUpdateBuilderTextField,
        update_value: impl Into<String>,
    ) -> Result<(), DbPostUpdateBuilderTextErr> {
        let pool = &self.db;

        let user_username = user_username.into();
        let update_value = update_value.into();

        let mut tx = pool.begin().await?;

        let query = "SELECT post_user_username FROM posts WHERE post_id = $1";

        let result = sqlx::query_as(query)
            .bind(post_id)
            .fetch_one(&mut *tx)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(DbPostUpdateBuilderTextErr::PostNotFound);
            }
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostUpdateBuilderTextErr::Db(err));
            }
        };

        debug!("query {query}\nresults {result:?}");

        let (post_user_username,): (String,) = result;

        if user_username != post_user_username {
            return Err(DbPostUpdateBuilderTextErr::Unauthorized);
        }

        let query_str = format!(
            "UPDATE posts SET
            {} = $1,
            post_modified_at = $2
            WHERE post_id = $3",
            update_field.as_str(),
        );
        let query = AssertSqlSafe(query_str.clone());

        debug!("about to run {query_str}");

        let result = sqlx::query(query)
            .bind(&update_value)
            .bind(XTimestamp(time as i64))
            .bind(post_id)
            .execute(&mut *tx)
            .await;

        let result = match result {
            Ok(v) => v,
            Err(err) => {
                error!("unexpected db error {err}");
                return Err(DbPostUpdateBuilderTextErr::Db(err));
            }
        };

        debug!("query {query_str}\nresults {result:?}");

        let affected = result.rows_affected();
        if affected != 1 {
            tx.rollback().await.unwrap();
            panic!(
                "something is wrong, updated wrong number of rows, expected 1, got {}, query {}, params {} {} {}",
                affected, query_str, update_value, time, post_id
            );
        }

        tx.commit().await?;

        Ok(())
    }
}
