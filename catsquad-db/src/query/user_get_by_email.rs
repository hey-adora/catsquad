use crate::{Db, DbUserGet, query::user_get_builder::DbUserGetBuilderErr};

#[derive(Debug, thiserror::Error)]
pub enum DbUserGetByEmailErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,
}

fn from_user_get_builder(value: DbUserGetBuilderErr) -> DbUserGetByEmailErr {
    match value {
        DbUserGetBuilderErr::NotFound => DbUserGetByEmailErr::NotFound,
        DbUserGetBuilderErr::Db(err) => DbUserGetByEmailErr::Db(err),
        DbUserGetBuilderErr::InvalidInput => unreachable!("invalid quary input"),
    }
}

impl Db {
    pub async fn user_get_by_email(
        &self,
        email: impl Into<String>,
    ) -> Result<DbUserGet, DbUserGetByEmailErr> {
        self.user_get_builder(String::new(), 0_u128.to_be_bytes(), String::new(), email)
            .await
            .map_err(from_user_get_builder)
    }
}
