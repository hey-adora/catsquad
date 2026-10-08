use crate::{Db, DbImageKind, XTimestamp};
use catsquad_log::prelude::*;
use sqlx::{PgConnection, Postgres, Transaction};

impl Db {
    pub async fn file_image_add_tx(
        &self,
        tx: &mut PgConnection,
        time: u64,
        file_hash: i64,
        file_size: u32,
        file_width: u32,
        file_height: u32,
        file_extension: &str,
    ) -> Result<(u32, u32), sqlx::Error> {
        let (image_exists, image_used_count, file_size) = {
            // let query = "SELECT EXISTS(SELECT 1 FROM files_images WHERE image_hash=$1)";
            let query =
                "SELECT image_used_count, image_size_bytes FROM files_images WHERE image_hash=$1";
            let result = sqlx::query_as(query)
                .bind(file_hash)
                .fetch_one(&mut *tx)
                .await;

            debug!("query {query}, result: {result:#?}");

            let result = match result {
                Ok(v) => {
                    let (image_usage_count, image_size_bytes): (i64, i64) = v;
                    (true, image_usage_count as u32 + 1, image_size_bytes as u32) // +1 because we will use this image next
                }
                Err(sqlx::Error::RowNotFound) => (false, 1, file_size), // 1 because we will create this img next
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(err);
                }
            };

            result
        };

        if !image_exists {
            // insert image
            {
                let query = "
            INSERT INTO files_images (
                    image_hash,
                    image_extension,
                    image_kind,
                    image_size_bytes,
                    image_width,
                    image_height,
                    image_modified_at,
                    image_created_at
                )
                VALUES ( $1, $2, $3, $4, $5, $6, $7, $7 )
            ";

                let result = sqlx::query(query)
                    .bind(&file_hash)
                    .bind(file_extension)
                    .bind(DbImageKind::Post.as_str())
                    .bind(file_size as i64)
                    .bind(file_width as i64)
                    .bind(file_height as i64)
                    .bind(XTimestamp(time as i64))
                    .execute(&mut *tx)
                    .await;

                debug!("query: {query}\nresult: {result:#?}");

                match result {
                    Ok(_v) => (),
                    Err(err) => {
                        error!("unexpected db error {err}");
                        return Err(err);
                    }
                };
            }
        } else {
            // image_used_count
            let query = "UPDATE files_images SET
                            image_used_count = image_used_count + 1,
                            image_modified_at = $2
                            WHERE image_hash = $1";

            let result = sqlx::query(query)
                .bind(file_hash)
                .bind(XTimestamp(time as i64))
                .execute(&mut *tx)
                .await;

            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("unexpected db error {err}");
                    return Err(err);
                }
            };

            debug!("query: {query}\nresult: {result:#?}");
        }

        Ok((image_used_count, file_size))
    }
}
