use std::fmt::Debug;

use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PostAddErr, validate_post_description, validate_post_tags, validate_post_title,
};
use leptos::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct FieldSaved {
    pub saved: bool,
    pub saved_at: u128,
    pub set_at: u128,
    pub checked_at: u128,
}

impl FieldSaved {
    pub fn new(time: u128) -> Self {
        Self {
            saved: true,
            saved_at: time,
            set_at: time,
            checked_at: time,
        }
    }
}

#[derive(Clone, Copy)]
pub struct UploadState {
    pub post_key: StoredValue<String>,
    pub title: RwSignal<String>,
    pub title_saved: RwSignal<FieldSaved>,
    pub description: RwSignal<String>,
    pub description_saved: RwSignal<FieldSaved>,
    pub tags: RwSignal<String>,
    pub tags_saved: RwSignal<FieldSaved>,
    pub err_general: RwSignal<String>,
    pub err_title: RwSignal<String>,
    pub err_description: RwSignal<String>,
    pub err_tags: RwSignal<String>,
}

impl UploadState {
    pub fn new(time: u128) -> Self {
        Self {
            post_key: StoredValue::new(String::new()),
            title: RwSignal::new(String::new()),
            title_saved: RwSignal::new(FieldSaved::new(time)),
            description: RwSignal::new(String::new()),
            description_saved: RwSignal::new(FieldSaved::new(time)),
            tags: RwSignal::new(String::new()),
            tags_saved: RwSignal::new(FieldSaved::new(time)),
            err_general: RwSignal::new(String::new()),
            err_title: RwSignal::new(String::new()),
            err_description: RwSignal::new(String::new()),
            err_tags: RwSignal::new(String::new()),
        }
    }

    pub async fn init<TSender>(&self, client: &Client<TSender>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let result = client
            .post_add("", "", "")
            .await
            .send()
            .await
            .into_res()
            .await;
        match result {
            Ok(v) => {
                if self.post_key.try_set_value(v.key).is_some() {
                    error!("page was disposed");
                    return;
                }

                if self.title.try_set(v.title).is_some() {
                    error!("page was disposed");
                    return;
                }

                if self.description.try_set(v.description).is_some() {
                    error!("page was disposed");
                    return;
                }

                if self.tags.try_set(v.tags).is_some() {
                    error!("page was disposed");
                    return;
                }
            }
            Err(PostAddErr::InvalidTitle(err)) => {
                self.err_title.set(err);
            }
            Err(PostAddErr::InvalidDescription(err)) => {
                self.err_description.set(err);
            }
            Err(PostAddErr::InvalidTags(err)) => {
                self.err_tags.set(err);
            }
            Err(err) => {
                self.err_general.set(err.to_string());
            }
        }
    }

    pub fn set_title(&self, time: u128, title: impl Into<String>) {
        let title = title.into();
        let title = title.trim();
        let result = validate_post_title(title);
        match result {
            Ok(_) => {
                self.title_saved.update(|v| {
                    v.saved = false;
                });
                self.err_title.update(|v| v.clear());
            }
            Err(err) => self.err_title.set(err),
        }
        self.title.set(title.to_string());
        self.title_saved.update(|v| {
            v.set_at = time;
        });
    }

    pub fn set_description(&self, time: u128, description: impl Into<String>) {
        let description = description.into();
        let description = description.trim();
        let result = validate_post_description(description);
        match result {
            Ok(_) => {
                self.description_saved.update(|v| {
                    v.saved = false;
                });
                self.err_description.update(|v| v.clear());
            }
            Err(err) => self.err_description.set(err),
        }
        self.description.set(description.to_string());
        self.description_saved.update(|v| {
            v.set_at = time;
        });
    }

    pub fn set_tags(&self, time: u128, tags: impl Into<String>) {
        let tags = tags.into();
        let tags = tags.trim();
        let result = validate_post_tags(tags);
        match result {
            Ok(_) => {
                self.tags_saved.update(|v| {
                    v.saved = false;
                });
                self.err_tags.update(|v| v.clear());
            }
            Err(err) => self.err_tags.set(err),
        }
        self.tags.set(tags.to_string());
        self.tags_saved.update(|v| {
            v.set_at = time;
        });
    }

    pub async fn update_title<TSender>(&self, time: u128, client: &Client<TSender>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_key = self.post_key.get_value();
        let title_field_metadata = self.title_saved.get_untracked();
        if title_field_metadata.saved {
            return;
        }
        let new_title = self.title.get_untracked();

        let result = client
            .post_update_title(&post_key, new_title)
            .await
            .send()
            .await
            .into_res()
            .await;

        match result {
            Ok(res) => {}
            Err(catsquad_shared::PostUpdateTitleErr::InvalidTitle(err)) => self.err_title.set(err),
            Err(err) => self.err_title.set(err.to_string()),
        }

        self.title_saved.update(|v| {
            v.saved_at = time;
            v.saved = true;
        });
    }

    pub async fn update_description<TSender>(&self, time: u128, client: &Client<TSender>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_key = self.post_key.get_value();
        let description_field_metadata = self.description_saved.get_untracked();
        if description_field_metadata.saved {
            return;
        }
        let new_description = self.description.get_untracked();

        let result = client
            .post_update_description(&post_key, new_description)
            .await
            .send()
            .await
            .into_res()
            .await;

        match result {
            Ok(res) => {}
            Err(catsquad_shared::PostUpdateDescriptionErr::InvalidDescription(err)) => {
                self.err_description.set(err)
            }
            Err(err) => self.err_description.set(err.to_string()),
        }

        self.description_saved.update(|v| {
            v.saved_at = time;
            v.saved = true;
        });
    }

    pub async fn update_tags<TSender>(&self, time: u128, client: &Client<TSender>)
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_key = self.post_key.get_value();
        let tags_field_metadata = self.tags_saved.get_untracked();
        if tags_field_metadata.saved {
            return;
        }
        let new_tags = self.tags.get_untracked();

        let result = client
            .post_update_tags(&post_key, new_tags)
            .await
            .send()
            .await
            .into_res()
            .await;

        match result {
            Ok(res) => {}
            Err(catsquad_shared::PostUpdateTagsErr::InvalidTags(err)) => self.err_tags.set(err),
            Err(err) => self.err_tags.set(err.to_string()),
        }

        self.tags_saved.update(|v| {
            v.saved_at = time;
            v.saved = true;
        });
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_upload_state() {
    use catsquad_api::{auth::create_auth_cookie_str, utils::rng_str};
    use catsquad_shared::{
        MAX_POST_DESCRIPTION_LENGTH, MAX_POST_TAGS_LENGTH, MAX_POST_TITLE_LENGTH,
    };
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;
    // server.state.set_time(0).await;

    let (_user1, session1) = server
        .user_add(
            "prime1",
            "prime1@heyadora.com",
            "235j4t49ngerigrog#IOTNOnfo",
        )
        .await;

    server
        .inject_header(header::COOKIE, create_auth_cookie_str(session1.clone()))
        .await;

    let upload = UploadState::new(0);

    assert_eq!(upload.post_key.get_value(), "");
    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.title.get_untracked(), "");

    assert_eq!(upload.description_saved.get_untracked().saved, true);
    assert_eq!(upload.description_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.description.get_untracked(), "");

    assert_eq!(upload.tags.get_untracked(), "");

    // server.state.set_time(1).await;
    upload.init(&server.client).await;

    // asserts that running update without set does nothing

    upload.update_title(1, &server.client).await;
    upload.update_description(1, &server.client).await;
    upload.update_tags(1, &server.client).await;

    assert!(!upload.post_key.get_value().is_empty());

    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.title_saved.get_untracked().set_at, 0);
    assert_eq!(upload.title.get_untracked(), "");

    assert_eq!(upload.description_saved.get_untracked().saved, true);
    assert_eq!(upload.description_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.description_saved.get_untracked().set_at, 0);
    assert_eq!(upload.description.get_untracked(), "");

    assert_eq!(upload.tags_saved.get_untracked().saved, true);
    assert_eq!(upload.tags_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.tags_saved.get_untracked().set_at, 0);
    assert_eq!(upload.tags.get_untracked(), "");

    // asserts set state change

    upload.set_title(1, "title1");
    upload.set_description(2, "description1");
    upload.set_tags(3, "tags1");

    assert_eq!(upload.title_saved.get_untracked().saved, false);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.title_saved.get_untracked().set_at, 1);
    assert_eq!(upload.title.get_untracked(), "title1");

    assert_eq!(upload.description_saved.get_untracked().saved, false);
    assert_eq!(upload.description_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.description_saved.get_untracked().set_at, 2);
    assert_eq!(upload.description.get_untracked(), "description1");

    assert_eq!(upload.tags_saved.get_untracked().saved, false);
    assert_eq!(upload.tags_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.tags_saved.get_untracked().set_at, 3);
    assert_eq!(upload.tags.get_untracked(), "tags1");

    // asserts update state change

    upload.update_title(3, &server.client).await;
    upload.update_description(4, &server.client).await;
    upload.update_tags(5, &server.client).await;

    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 3);
    assert_eq!(upload.title_saved.get_untracked().set_at, 1);
    assert_eq!(upload.title.get_untracked(), "title1");
    assert_eq!(upload.err_title.get_untracked(), "");

    assert_eq!(upload.description_saved.get_untracked().saved, true);
    assert_eq!(upload.description_saved.get_untracked().saved_at, 4);
    assert_eq!(upload.description_saved.get_untracked().set_at, 2);
    assert_eq!(upload.description.get_untracked(), "description1");
    assert_eq!(upload.err_description.get_untracked(), "");

    assert_eq!(upload.tags_saved.get_untracked().saved, true);
    assert_eq!(upload.tags_saved.get_untracked().saved_at, 5);
    assert_eq!(upload.tags_saved.get_untracked().set_at, 3);
    assert_eq!(upload.tags.get_untracked(), "tags1");
    assert_eq!(upload.err_tags.get_untracked(), "");

    // asserts that invalid value doesnt get pushed

    let title = rng_str(MAX_POST_TITLE_LENGTH + 1);
    let description = rng_str(MAX_POST_DESCRIPTION_LENGTH + 1);
    let tags = rng_str(MAX_POST_TAGS_LENGTH + 1);

    upload.set_title(5, title);
    upload.set_description(6, description);
    upload.set_tags(7, tags);

    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 3);
    assert_eq!(upload.title_saved.get_untracked().set_at, 5);

    assert_eq!(upload.description_saved.get_untracked().saved, true);
    assert_eq!(upload.description_saved.get_untracked().saved_at, 4);
    assert_eq!(upload.description_saved.get_untracked().set_at, 6);

    assert_eq!(upload.tags_saved.get_untracked().saved, true);
    assert_eq!(upload.tags_saved.get_untracked().saved_at, 5);
    assert_eq!(upload.tags_saved.get_untracked().set_at, 7);

    upload.update_title(7, &server.client).await;
    upload.update_description(8, &server.client).await;
    upload.update_tags(9, &server.client).await;

    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 3);
    assert_eq!(upload.title_saved.get_untracked().set_at, 5);
    assert_ne!(upload.title.get_untracked(), "title1");
    assert!(!upload.err_title.get_untracked().is_empty());

    assert_eq!(upload.description_saved.get_untracked().saved, true);
    assert_eq!(upload.description_saved.get_untracked().saved_at, 4);
    assert_eq!(upload.description_saved.get_untracked().set_at, 6);
    assert_ne!(upload.description.get_untracked(), "description1");
    assert!(!upload.err_description.get_untracked().is_empty());

    assert_eq!(upload.tags_saved.get_untracked().saved, true);
    assert_eq!(upload.tags_saved.get_untracked().saved_at, 5);
    assert_eq!(upload.tags_saved.get_untracked().set_at, 7);
    assert_ne!(upload.tags.get_untracked(), "tags1");
    assert!(!upload.err_tags.get_untracked().is_empty());
}
