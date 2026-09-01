use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct DbFileImage {
    #[sqlx(rename = "image_hash")]
    pub hash: i64,
    #[sqlx(rename = "image_extension")]
    pub extension: String,
    #[sqlx(rename = "image_kind")]
    pub kind: String,
    #[sqlx(rename = "image_size_bytes")]
    #[sqlx(try_from = "i64")]
    pub size_bytes: u32,
    // #[sqlx(rename = "image_post_id")]
    // pub post_id: i64,
    // #[sqlx(rename = "image_user_username")]
    // pub user_username: String,
    #[sqlx(rename = "image_processed")]
    pub processed: bool,
    #[sqlx(rename = "image_used_count")]
    #[sqlx(try_from = "i64")]
    pub used_count: u32,
    // #[sqlx(rename = "image_state")]
    // pub state: String,
    #[sqlx(rename = "image_width")]
    #[sqlx(try_from = "i64")]
    pub width: u32,
    #[sqlx(rename = "image_height")]
    #[sqlx(try_from = "i64")]
    pub height: u32,
    #[sqlx(rename = "image_modified_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub modified_at: u64,
    #[sqlx(rename = "image_created_at")]
    #[sqlx(try_from = "XTimestamp")]
    pub created_at: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum DbImageKind {
    Post,
    Pfp,
}

impl DbImageKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DbImageKind::Post => "post",
            DbImageKind::Pfp => "pfp",
        }
    }
}

// #[derive(Clone, Copy, Debug)]
// pub enum DbImageState {
//     Active,
//     Hidden,
// }

// impl DbImageState {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             DbImageState::Active => "active",
//             DbImageState::Hidden => "hidden",
//         }
//     }
// }

impl Db {
    pub async fn file_image_define(&self) {
        let pool = &self.db;
        let query = "
            CREATE TABLE files_images (
                image_hash int8 PRIMARY KEY,
                image_extension varchar(10) NOT NULL,
                image_kind varchar(10) NOT NULL,
                image_size_bytes int8 NOT NULL,
                image_processed bool DEFAULT FALSE,
                image_used_count int8 DEFAULT 1,
                image_width int8 NOT NULL,
                image_height int8 NOT NULL,
                image_modified_at timestamp NOT NULL,
                image_created_at timestamp NOT NULL
            );
        ";
        // image_post_id int8 NOT NULL references posts(post_id),
        // image_user_username varchar(32) NOT NULL references users(user_username),
        // image_state varchar(10) DEFAULT 'hidden',
        trace!("about to run {query}");
        let _result = sqlx::raw_sql(query).execute(pool).await.unwrap();
    }
}
