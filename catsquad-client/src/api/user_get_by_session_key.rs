use catsquad_shared::{
    LINK_API_SESSION_GET_BY_SESSION_KEY, UserGetBySessionKeyErr, UserGetBySessionKeyRes,
};

use crate::get;

pub async fn get_user_by_session_key() -> Result<UserGetBySessionKeyRes, UserGetBySessionKeyErr> {
    get(LINK_API_SESSION_GET_BY_SESSION_KEY)
        .await
        .map_err(|_err| UserGetBySessionKeyErr::InternalServer)?
}
