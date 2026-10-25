use crate::{Db, DbUserGet, query::user_get_builder::DbUserGetBuilderErr};
use catsquad_shared::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetBySessionErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

fn from_user_get_builder(value: DbUserGetBuilderErr) -> DbUserGetBySessionErr {
    match value {
        DbUserGetBuilderErr::NotFound => DbUserGetBySessionErr::NotFound,
        DbUserGetBuilderErr::Db(err) => DbUserGetBySessionErr::Db(err),
        DbUserGetBuilderErr::InvalidInput => unreachable!("invalid quary input"),
    }
}

impl Db {
    pub async fn user_get_by_session(
        &self,
        user_username: impl Into<String>,
        session: Uuid,
    ) -> Result<DbUserGet, DbUserGetBySessionErr> {
        self.user_get_builder(user_username, session, String::new(), String::new())
            .await
            .map_err(from_user_get_builder)
    }
}
