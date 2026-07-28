use catsquad_shared::{InviteAddErr, InviteAddReq, InviteAddRes, LINK_API_INVITE_ADD};

use crate::post;

pub async fn post_invite_add(email: impl Into<String>) -> Result<InviteAddRes, InviteAddErr> {
    let req = InviteAddReq {
        email: email.into(),
    };
    post(LINK_API_INVITE_ADD, req)
        .await
        .map_err(|_err| InviteAddErr::InternalServer)
}
