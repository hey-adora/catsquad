use crate::{Db, DbUserGet, query::user_get_builder::DbUserGetBuilderErr};
use catsquad_shared::{SessionTokenKind, Uuid};

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetBySessionErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized(String),

    #[error("internal server error")]
    InternalServerErr,
}

fn from_user_get_builder(value: DbUserGetBuilderErr) -> DbUserGetBySessionErr {
    match value {
        DbUserGetBuilderErr::Unauthorized(err) => DbUserGetBySessionErr::Unauthorized(err),
        DbUserGetBuilderErr::InternalServerErr => DbUserGetBySessionErr::InternalServerErr,
        DbUserGetBuilderErr::NotFound => DbUserGetBySessionErr::NotFound,
        DbUserGetBuilderErr::Db(err) => DbUserGetBySessionErr::Db(err),
        DbUserGetBuilderErr::InvalidInput => unreachable!("invalid quary input"),
    }
}

impl Db {
    pub async fn user_get_by_session(
        &self,
        auth_session_token: SessionTokenKind,
        // user_username: impl Into<String>,
        session: Uuid,
    ) -> Result<DbUserGet, DbUserGetBySessionErr> {
        self.user_get_builder(auth_session_token, session, String::new(), String::new())
            .await
            .map_err(from_user_get_builder)
    }
}
