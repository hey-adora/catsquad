pub const TEST_BACKDOOR_LINK_API_EMAIL_SENT_GET_ALL: &str = "/api/test_backdoor_email_sent_get_all";

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct EmailSentRes {
//     pub em: String,
//     pub username: String,
//     pub email: String,
//     pub created_at: u128,
// }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EmailSentRes {
    pub body: String,
    pub to_email: String,
    pub reason: String,
    pub created_at: u128,
}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
// pub struct PostGetByKeyParams {
//     pub post_key: String,
// }
// pub const POST_GET_BY_KEY_REQ_FIELD_POST_KEY: &'static str = "post_key";

#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, thiserror::Error,
)]
pub enum TestBackdoorEmailSentGetAllErr {
    #[default]
    #[error("internal server err")]
    InternalServerErr,
}
