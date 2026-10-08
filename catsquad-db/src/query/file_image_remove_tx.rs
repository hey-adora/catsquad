use crate::{Db, XTimestamp};
use catsquad_log::prelude::*;
use catsquad_shared::i64_to_str;
use sqlx::PgConnection;

#[derive(Debug, thiserror::Error)]
pub enum DbFileImageRemoveTxErr {
    #[error("DB error {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal error {0}")]
    InternalError(String),
}

impl Db {
    /// careful, can only be used to undo file_image_add_tx
    /// returns (use_count, image_size)
    pub async fn file_image_remove_tx(
        &self,
        tx: &mut PgConnection,
        time: u64,
        file_hash: i64,
    ) -> Result<(u32, u32), DbFileImageRemoveTxErr> {
        // get img
        let (mut img_used_count, file_size) = {
            let query =
                "SELECT image_used_count, image_size_bytes FROM files_images WHERE image_hash=$1";
            let result = sqlx::query_as(query)
                .bind(file_hash)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query}, result: {result:#?}");

            let result = match result {
                Ok(v) => {
                    let (image_used_count, image_size_bytes): (i64, i64) = v;
                    if image_used_count < 1 {
                        return Err(DbFileImageRemoveTxErr::InternalError(format!(
                            "{image_used_count}(image_used_count) < 1 for image {file_hash}"
                        )));
                    }
                    (image_used_count as u32, image_size_bytes as u32)
                }
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFileImageRemoveTxErr::Db(err));
                }
            };

            result
        };

        if img_used_count == 1 {
            // remove image
            let query = "DELETE FROM files_images WHERE image_hash = $1";
            let result = sqlx::query(query).bind(file_hash).execute(&mut *tx).await;

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFileImageRemoveTxErr::Db(err));
                }
            };

            if result.rows_affected() != 1 {
                return Err(DbFileImageRemoveTxErr::InternalError(
                    "rows affected != 1".to_string(),
                ));
            }
        } else {
            // decrement used_count
            let query = "UPDATE files_images SET
                            image_used_count =  image_used_count - 1,
                            image_modified_at = $1
                            WHERE image_hash = $2";

            let result = sqlx::query(query)
                .bind(XTimestamp(time as i64))
                .bind(file_hash)
                .execute(&mut *tx)
                .await;

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(DbFileImageRemoveTxErr::Db(err));
                }
            };

            if result.rows_affected() != 1 {
                return Err(DbFileImageRemoveTxErr::InternalError(
                    "rows affected != 1".to_string(),
                ));
            }
        }

        img_used_count -= 1; // because we removed it

        Ok((img_used_count, file_size))
    }
}
