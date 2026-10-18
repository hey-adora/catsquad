use crate::ProfileParams;
use catsquad_client::{Client, Response, Sender, XMLSender};
use catsquad_log::prelude::*;
use catsquad_shared::{
    MATCH_FOLLOWED, MATCH_INDEX, MATCH_PROFILE, MATCH_PROFILE_ABOUTME, MATCH_PROFILE_PAWS,
    MATCH_PROFILE_SUPPORT, SensitiveUserRes, SessionRemoveErr, UserGetBySessionKeyErr,
};
use catsquad_web_utils::time::time_now_micro;
use leptos::prelude::*;
use leptos_router::{hooks::use_params, location::Location};
use std::fmt::Debug;
use strum::EnumIs;

// pub acc: RwSignal<Option<SensitiveUserRes>>,
#[derive(Clone, Debug)]
pub struct PageState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub client: StoredValue<Client<TSender>, LocalStorage>,
    pub acc_username: RwSignal<String>,
    pub acc_pfp_hash: RwSignal<i64>,
    pub acc_pfp_proccesed: RwSignal<bool>,
    pub acc_max_storage_bytes: RwSignal<u32>, // TODO add max per image and add that to UI
    pub acc_email: RwSignal<String>,
    pub acc_created_at: RwSignal<u64>,
    pub is_login_pending: RwSignal<bool>,
    pub is_logged_in: RwSignal<bool>,
    pub time: RwSignal<u64>,
}

impl<TSender> Copy for PageState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
}

#[derive(Clone, PartialEq, PartialOrd, EnumIs)]
pub enum CurrentRoute {
    Index,
    Followed,
    ProfileGallery,
    ProfilePaws,
    ProfileSupport,
    ProfileAboutMe,
    Other,
}

impl PageState<XMLSender> {
    pub fn get() -> Self {
        expect_context::<Self>()
    }
}

impl<TSender> PageState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            client: StoredValue::new_local(client),
            time: RwSignal::new(time_now_micro()),
            acc_username: RwSignal::new(String::new()),
            acc_pfp_hash: RwSignal::new(0),
            acc_pfp_proccesed: RwSignal::new(false),
            acc_max_storage_bytes: RwSignal::new(0), // TODO add max per image and add that to UI
            acc_email: RwSignal::new(String::new()),
            acc_created_at: RwSignal::new(0),
            is_login_pending: RwSignal::new(true),
            is_logged_in: RwSignal::new(false),
        }
    }

    pub fn current_route(&self, location: Location) -> Memo<CurrentRoute> {
        Memo::new(move |_| {
            let query = location.search.get();
            let pathname = location.pathname.get();

            if pathname == MATCH_INDEX {
                return CurrentRoute::Index;
            }

            if pathname == MATCH_FOLLOWED {
                return CurrentRoute::Followed;
            }

            if pathname.contains(MATCH_PROFILE) {
                if query.contains(MATCH_PROFILE_PAWS) {
                    return CurrentRoute::ProfilePaws;
                }
                if query.contains(MATCH_PROFILE_ABOUTME) {
                    return CurrentRoute::ProfileAboutMe;
                }
                if query.contains(MATCH_PROFILE_SUPPORT) {
                    return CurrentRoute::ProfileSupport;
                }
                return CurrentRoute::ProfileGallery;
            }

            CurrentRoute::Other
        })
    }

    pub fn create_page_profile_params_hook(&self) -> Memo<ProfileParams> {
        let params = use_params::<ProfileParams>();
        Memo::new(move |_| params.get().ok().unwrap_or_default())
    }

    pub fn set(client: Client<TSender>) {
        provide_context(Self::new(client));
    }

    pub fn get_time(&self) -> u64 {
        self.time.get()
    }

    pub fn is_logged_in(&self) -> Option<bool> {
        let is_pending = self.is_login_pending.get();
        let is_logged_in = self.is_logged_in.get();
        if is_pending {
            return None;
        }
        Some(is_logged_in)
    }

    pub fn acc_pending(&self) -> bool {
        self.is_login_pending.get()
    }

    pub fn get_pfp_hash(&self) -> i64 {
        self.acc_pfp_hash.get()
    }

    pub fn get_pfp_proccesed_status(&self) -> bool {
        self.acc_pfp_proccesed.get()
    }

    pub fn acc_max_storage_bytes(&self) -> u32 {
        self.acc_max_storage_bytes.get()
    }

    pub fn acc_username(&self) -> String {
        self.acc_username.get()
    }

    pub fn acc_username_set(&self, new_username: impl Into<String>) {
        self.acc_username.set(new_username.into())
    }

    pub fn acc_email_set(&self, new_email: impl Into<String>) {
        self.acc_email.set(new_email.into());
    }

    pub fn acc_email(&self) -> String {
        self.acc_email.get()
    }

    pub fn acc_clear(&self) {
        self.acc_username.try_update(|v| v.clear());
        self.acc_email.try_update(|v| v.clear());
        self.acc_pfp_hash.try_set(0);
        self.acc_pfp_proccesed.try_set(false);
        self.acc_max_storage_bytes.try_set(0);
        self.acc_created_at.try_set(0);
        self.is_login_pending.try_set(false);
        self.is_logged_in.try_set(false);
    }

    pub async fn update_auth(&self) {
        let page = self;
        let client = self.client.get_value();
        let result = client
            .user_get_by_session_id()
            .send()
            .await
            .into_json()
            .await;
        match result {
            Ok(user) => {
                page.acc_username.set(user.username);
                page.acc_pfp_hash.set(user.pfp_hash);
                page.acc_pfp_proccesed.set(user.pfp_proccesed);
                page.acc_email.set(user.email);
                page.acc_max_storage_bytes.set(user.max_storage_bytes);
                page.acc_created_at.set(user.created_at);
                page.is_login_pending.set(true);
                page.is_logged_in.set(true);
            }
            Err(UserGetBySessionKeyErr::Unauthorized(_)) => {
                page.acc_clear();
                info!("user is not logged in")
            }
            Err(UserGetBySessionKeyErr::InternalServer) => {
                error!("acc update internal server err")
            }
        }
        page.is_login_pending.set(false);
    }

    // pub async fn update_pfp_proccesed_status(&self) {
    //     let client = self.client.get_value();
    //     let (hash, is_proccesed) = self
    //         .acc
    //         .with_untracked(|v| v.as_ref().map(|v| (v.pfp_hash, v.pfp_proccesed)))
    //         .unwrap_or_default();

    //     if is_proccesed || hash == 0 {
    //         warn!("pfp is already updated");
    //         return;
    //     }

    //     let result = client
    //         .file_image_status_get_by_hash(hash)
    //         .send()
    //         .await
    //         .into_json()
    //         .await;

    //     match result {
    //         Ok(v) => {
    //             self.acc.update(|user| {
    //                 if let Some(user) = user {
    //                     user.pfp_proccesed = v.is_proccesed;
    //                 }
    //             });
    //         }
    //         Err(err) => {
    //             error!("pfp update internal server err {err}")
    //         }
    //     }
    // }

    pub async fn logout(&self) {
        if self.is_logged_in.get_untracked() {
            let page_state = *self;
            let Some(client) = self.client.try_get_value() else {
                return;
            };
            let result = client.session_remove().send().await.into_json().await;
            match result {
                Ok(_) => {
                    page_state.acc_clear();
                }
                Err(SessionRemoveErr::Unauthorized(_)) => {
                    page_state.acc_clear();
                }
                Err(err) => error!("logout fail"),
            }
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_page_state() {
    use catsquad_api::{auth::create_auth_cookie_str, proccess_images::proccess_images_all};
    use catsquad_shared::uuid_to_str;
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_page_state").await;
    let input1_img = server.create_img_input(0).await;
    let storage_path = server.state.get_storage_path().await;

    let (_user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    let page = PageState::new(server.client.clone());
    assert_eq!(page.get_pfp_hash(), 0);
    assert_eq!(page.get_pfp_proccesed_status(), false);
    assert_eq!(page.acc_pending(), true);
    assert_eq!(page.acc_username(), "");
    assert_eq!(page.acc_email(), "");
    assert_eq!(page.is_logged_in(), None);

    // page.update_pfp_proccesed_status().await;

    // assert_eq!(page.get_pfp_hash(), 0);
    // assert_eq!(page.get_pfp_proccesed_status(), false);
    // assert_eq!(page.acc_pending(), true);
    // assert_eq!(page.acc_username(), "");
    // assert_eq!(page.acc_email(), "");
    // assert_eq!(page.is_logged_in(), None);

    page.update_auth().await;

    assert_eq!(page.get_pfp_hash(), 0);
    assert_eq!(page.get_pfp_proccesed_status(), false);
    assert_eq!(page.acc_pending(), false);
    assert_eq!(page.acc_username(), "");
    assert_eq!(page.acc_email(), "");
    assert_eq!(page.is_logged_in(), Some(false));

    server
        .inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session1)),
        )
        .await;

    page.update_auth().await;

    assert_eq!(page.get_pfp_hash(), 0);
    assert_eq!(page.get_pfp_proccesed_status(), false);
    assert_eq!(page.acc_pending(), false);
    assert_eq!(page.acc_username(), "prime1");
    assert_eq!(page.acc_email(), "prime1@heyadora.com");
    assert_eq!(page.is_logged_in(), Some(true));

    // page.update_pfp_proccesed_status().await;

    // assert_eq!(page.get_pfp_hash(), 0);
    // assert_eq!(page.get_pfp_proccesed_status(), false);

    server
        .user_update_pfp(&[input1_img.saved_path.to_str().unwrap()], session1)
        .await
        .unwrap();

    // page.update_pfp_proccesed_status().await;

    // assert_eq!(page.get_pfp_hash(), 0);
    // assert_eq!(page.get_pfp_proccesed_status(), false);

    page.update_auth().await;

    assert_eq!(page.get_pfp_hash(), input1_img.hash);
    assert_eq!(page.get_pfp_proccesed_status(), false);
    assert_eq!(page.acc_pending(), false);
    assert_eq!(page.acc_username(), "prime1");
    assert_eq!(page.acc_email(), "prime1@heyadora.com");
    assert_eq!(page.is_logged_in(), Some(true));

    page.logout().await;

    assert_eq!(page.get_pfp_hash(), 0);
    assert_eq!(page.get_pfp_proccesed_status(), false);
    assert_eq!(page.acc_pending(), false);
    assert_eq!(page.acc_username(), "");
    assert_eq!(page.acc_email(), "");
    assert_eq!(page.is_logged_in(), Some(false));

    // page.update_pfp_proccesed_status().await;

    // assert_eq!(page.get_pfp_proccesed_status(), false);

    // proccess_images_all(0, server.state.db.clone(), storage_path, 1280)
    //     .await
    //     .unwrap();

    // page.update_pfp_proccesed_status().await;

    // assert_eq!(page.get_pfp_proccesed_status(), true);
}
