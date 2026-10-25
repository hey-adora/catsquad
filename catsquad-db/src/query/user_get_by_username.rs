use crate::{Db, XTimestamp, query::user_get_builder::DbUserGetBuilderErr};

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

fn from_user_get_builder(value: DbUserGetBuilderErr) -> DbUserGetByUsernameErr {
    match value {
        DbUserGetBuilderErr::NotFound => DbUserGetByUsernameErr::NotFound,
        DbUserGetBuilderErr::Db(err) => DbUserGetByUsernameErr::Db(err),
        DbUserGetBuilderErr::InvalidInput => unreachable!("invalid quary input"),
    }
}

impl Db {
    pub async fn user_get_by_username(
        &self,
        user_username: impl Into<String>,
        username: impl Into<String>,
    ) -> Result<DbUserGet, DbUserGetByUsernameErr> {
        self.user_get_builder(user_username, 0_u128.to_be_bytes(), username, String::new())
            .await
            .map_err(from_user_get_builder)
    }
}
