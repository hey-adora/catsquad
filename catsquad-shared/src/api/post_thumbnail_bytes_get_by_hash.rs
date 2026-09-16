use catsquad_log::prelude::*;

pub const LINK_API_POST_THUMBNAIL_BYTES_GET_BY_HASH: &str =
    "/api/post/{post_id}/file/{file_hash}/thumbnail";

pub fn link_relative_post_thumbnail_bytes_get_by_hash(post_id: i64, file_hash: i64) -> String {
    format!("/api/post/{}/file/{}/thumbnail", post_id, file_hash)
}
