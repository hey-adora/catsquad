use catsquad_client::{Client, Response, SchrodingersImage, Sender};
use catsquad_log::prelude::*;
use leptos::prelude::*;
use std::fmt::Debug;

pub struct UserPfpState<TSender>
where
    TSender: Sender + Debug + Clone,
    TSender::TResponse: Response + Debug,
{
    pub err_general: RwSignal<String>,
    pub pfp_hash: RwSignal<i64>,
    pub pfp_proccesed_status: RwSignal<bool>,
    pub client: StoredValue<Client<TSender>, LocalStorage>,
}

impl<TSender> Clone for UserPfpState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    fn clone(&self) -> Self {
        Self {
            err_general: self.err_general.clone(),
            pfp_hash: self.pfp_hash.clone(),
            pfp_proccesed_status: self.pfp_proccesed_status.clone(),
            client: self.client.clone(),
        }
    }
}

impl<TSender> Copy for UserPfpState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
}

impl<TSender> UserPfpState<TSender>
where
    TSender: Sender + Debug + Clone + 'static,
    TSender::TResponse: Response + Debug,
{
    pub fn new(client: Client<TSender>) -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            pfp_hash: RwSignal::new(0),
            pfp_proccesed_status: RwSignal::new(false),
            client: StoredValue::new_local(client),
        }
    }

    pub fn new_with_state(
        client: Client<TSender>,
        pfp_hash: RwSignal<i64>,
        pfp_proccesed_status: RwSignal<bool>,
    ) -> Self {
        Self {
            err_general: RwSignal::new(String::new()),
            pfp_hash: pfp_hash,
            pfp_proccesed_status: pfp_proccesed_status,
            client: StoredValue::new_local(client),
        }
    }

    pub async fn update_pfp_proccesed_status(&self) {
        let client = self.client.get_value();
        let hash = self.pfp_hash.get_untracked();
        let is_proccesed = self.pfp_proccesed_status.get_untracked();

        if is_proccesed || hash == 0 {
            warn!("pfp is already updated");
            return;
        }

        let result = client
            .file_image_status_get_by_hash(hash)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {
                self.pfp_proccesed_status.set(v.is_proccesed);
            }
            Err(err) => {
                error!("pfp update internal server err {err}")
            }
        }
    }

    pub async fn update_pfp<TImage>(&self, image: TImage)
    where
        TImage: Into<SchrodingersImage>,
    {
        let client = self.client.get_value();

        let result = client
            .user_update_pfp(vec![image])
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {
                self.pfp_hash.set(v.hash);
                self.pfp_proccesed_status.set(v.proccesed);
            }
            Err(err) => {
                error!("pfp update internal server err {err}")
            }
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_user_pfp_state() {
    use catsquad_api::{auth::create_auth_cookie_str, proccess_images::proccess_images_all};
    use catsquad_shared::{FILE_IMAGE_THUMBNAIL_SIZE, uuid_to_str};
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, "test_username_change_state").await;
    let input1_img = server.create_img_input(0).await;
    let storage_path = server.state.get_storage_path().await;

    let (_user1, session1) = server
        .user_add_full(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    server
        .inject_header(
            header::COOKIE,
            create_auth_cookie_str(uuid_to_str(session1)),
        )
        .await;

    let pfp_state = UserPfpState::new(server.client.clone());

    assert_eq!(pfp_state.pfp_hash.get_untracked(), 0);
    assert_eq!(pfp_state.pfp_proccesed_status.get_untracked(), false);

    pfp_state
        .update_pfp(input1_img.saved_path.to_str().unwrap())
        .await;

    assert_eq!(pfp_state.pfp_hash.get_untracked(), input1_img.hash);
    assert_eq!(pfp_state.pfp_proccesed_status.get_untracked(), false);

    pfp_state.update_pfp_proccesed_status().await;

    assert_eq!(pfp_state.pfp_hash.get_untracked(), input1_img.hash);
    assert_eq!(pfp_state.pfp_proccesed_status.get_untracked(), false);

    proccess_images_all(0, server.state.db, storage_path, FILE_IMAGE_THUMBNAIL_SIZE)
        .await
        .unwrap();

    pfp_state.update_pfp_proccesed_status().await;

    assert_eq!(pfp_state.pfp_hash.get_untracked(), input1_img.hash);
    assert_eq!(pfp_state.pfp_proccesed_status.get_untracked(), true);
}
