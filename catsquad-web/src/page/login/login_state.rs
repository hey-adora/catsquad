use leptos::prelude::*;

// #[derive(Clone, Copy)]
// pub struct ChangePassword {
//     pub err_general: RwSignal<String>,
//     // pub email: RwQuery<String>,
//     // pub form_stage: RwQuery<ChangePasswordFormStage>,
//     // pub btn_stage: StoredValue<Box<dyn Fn() -> ChangePasswordBtnStage + Sync + Send + 'static>>,
//     // pub on_change: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
// }

// #[cfg(test)]
// pub mod tests {
//     use crate::init_owner;
//     use catsquad_api::TestServer;
//     use catsquad_client::api_invite_add;
//     use catsquad_log::prelude::*;

//     #[tokio::test]
//     async fn test_login() {
//         init_log();
//         let owner = init_owner();
//         let server = TestServer::new().await;

//         trace!("wtf is going on");

//         let result = api_invite_add("hey@heyadora.com").await;
//         // trace!("aaaaaaaa: {result:?}");
//     }
// }
