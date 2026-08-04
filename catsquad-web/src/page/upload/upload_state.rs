use std::fmt::Debug;

use catsquad_client::{Client, Response, SchrodingersFile, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{
    PostAddErr, PostFile, PostState, link_relative_post, validate_post_description,
    validate_post_tags, validate_post_title,
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

#[derive(Clone)]
pub struct ParsedPostFile {
    pub name: String,
    pub size: u64,
    pub uploaded_bytes: u64,
    pub uploaded_percentage: u64,
    pub upload_speed_bytes_a_second: u64,
    pub state: ParsedPostFileState,
    pub err: String,
}

impl From<String> for ParsedPostFile {
    fn from(value: String) -> Self {
        Self {
            name: value,
            size: 0,
            uploaded_bytes: 0,
            upload_speed_bytes_a_second: 0,
            uploaded_percentage: 0,
            state: ParsedPostFileState::Queue,
            err: String::new(),
        }
    }
}

impl From<&str> for ParsedPostFile {
    fn from(value: &str) -> Self {
        From::<String>::from(value.to_string())
    }
}

impl From<PostFile> for ParsedPostFile {
    fn from(value: PostFile) -> Self {
        Self {
            name: value.hash,
            size: value.size_bytes,
            uploaded_bytes: 0,
            upload_speed_bytes_a_second: 0,
            uploaded_percentage: 0,
            state: match value.proccesed {
                true => ParsedPostFileState::Proccesed,
                false => ParsedPostFileState::Uploaded,
            },
            err: String::new(),
        }
    }
}

impl From<web_sys::File> for ParsedPostFile {
    fn from(file: web_sys::File) -> Self {
        let name = file.name();
        let size = file.size();
        Self {
            name: name,
            size: size as u64,
            uploaded_bytes: 0,
            upload_speed_bytes_a_second: 0,
            uploaded_percentage: 0,
            state: ParsedPostFileState::Queue,
            err: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParsedPostFileState {
    Queue,
    Uploading,
    Uploaded,
    Proccesed,
    Error,
    Removing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UploadStateStage {
    Activating,
    Loading,
    Loaded,
    Err,
}

#[derive(Clone, Copy)]
pub struct UploadState {
    pub post_key: StoredValue<String>,
    pub stage: RwSignal<UploadStateStage>,
    pub title: RwSignal<String>,
    pub title_saved: RwSignal<FieldSaved>,
    pub description: RwSignal<String>,
    pub description_saved: RwSignal<FieldSaved>,
    pub tags: RwSignal<String>,
    pub tags_saved: RwSignal<FieldSaved>,
    pub files: RwSignal<Vec<ArcRwSignal<ParsedPostFile>>>,
    pub err_general: RwSignal<String>,
    pub err_title: RwSignal<String>,
    pub err_description: RwSignal<String>,
    pub err_tags: RwSignal<String>,
}

impl UploadState {
    pub fn new(time: u128) -> Self {
        Self {
            post_key: StoredValue::new(String::new()),
            stage: RwSignal::new(UploadStateStage::Loading),
            title: RwSignal::new(String::new()),
            title_saved: RwSignal::new(FieldSaved::new(time)),
            description: RwSignal::new(String::new()),
            description_saved: RwSignal::new(FieldSaved::new(time)),
            tags: RwSignal::new(String::new()),
            tags_saved: RwSignal::new(FieldSaved::new(time)),
            files: RwSignal::new(Vec::new()),
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
        let result = client.post_add("", "", "").send().await.into_res().await;
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

                if self
                    .files
                    .try_set(
                        v.file
                            .into_iter()
                            .map(|v| ArcRwSignal::new(ParsedPostFile::from(v)))
                            .collect(),
                    )
                    .is_some()
                {
                    error!("page was disposed");
                    return;
                }

                if self.stage.try_set(UploadStateStage::Loaded).is_some() {
                    error!("page was disposed");
                    return;
                }
            }
            Err(PostAddErr::InvalidTitle(err)) => {
                self.err_title.try_set(err);
            }
            Err(PostAddErr::InvalidDescription(err)) => {
                self.err_description.try_set(err);
            }
            Err(PostAddErr::InvalidTags(err)) => {
                self.err_tags.try_set(err);
            }
            Err(err) => {
                self.stage.try_set(UploadStateStage::Err);
                self.err_general.try_set(err.to_string());
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

    pub async fn update_state_active<TSender>(&self, client: &Client<TSender>) -> Option<String>
    where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_key = self.post_key.get_value();

        if post_key.is_empty() {
            warn!("trying to set state while upload isn't initialized");
            return None;
        }

        let result = client
            .post_update_state(&post_key, PostState::Active)
            .send()
            .await
            .into_res()
            .await;

        match result {
            Ok(v) => {
                return Some(link_relative_post(v.key));
            }
            Err(v) => {
                self.err_general.set(v.to_string());
                return None;
            }
        }
    }

    pub fn set_files<I>(&self, files: I) -> Vec<ArcRwSignal<ParsedPostFile>>
    where
        I: IntoIterator + Clone,
        I::Item: Into<ParsedPostFile>,
    {
        let files_signals = files
            .into_iter()
            .map(|v| ArcRwSignal::new(Into::<ParsedPostFile>::into(v)))
            .collect::<Vec<ArcRwSignal<ParsedPostFile>>>();

        self.files.update({
            let files_signals = files_signals.clone();
            |parsed_files| {
                parsed_files.extend(files_signals);
            }
        });

        files_signals
    }

    pub async fn update_file<TSender, File>(
        &self,
        client: &Client<TSender>,
        source_file: File,
        parsed_file: ArcRwSignal<ParsedPostFile>,
    ) where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
        File: Into<SchrodingersFile>,
    {
        let post_key = self.post_key.try_get_value().unwrap_or_default();
        if post_key.is_empty() {
            warn!("trying upload files when post wasn't initialized");
            return;
        };

        let result = client
            .post_update_file_add(post_key, vec![source_file])
            .on_progress({
                let file = parsed_file.clone();
                move |stats| {
                    trace!("UPLOADING {stats:?}");
                    file.update(|parsed_file| {
                        parsed_file.state = ParsedPostFileState::Uploading;
                        parsed_file.uploaded_bytes = stats.completed_bytes;
                        parsed_file.upload_speed_bytes_a_second = stats.upload_speed_bytes;
                        parsed_file.uploaded_percentage = stats.completed_precentage;
                    });
                }
            })
            .send()
            .await
            .into_res()
            .await;

        match result {
            Ok(received_post) => {
                if received_post.len() != 1 {
                    warn!("received wrong data\n{received_post:#?}");
                    parsed_file.update(|file| {
                        file.state = ParsedPostFileState::Error;
                        file.err = "received wrong response".to_string();
                    });
                    return;
                }

                let hash = received_post[0].hash.clone();

                parsed_file.update(|file| {
                    file.name = hash;
                    file.state = ParsedPostFileState::Uploaded;
                });
            }
            Err(err) => {
                parsed_file.update(|file| {
                    file.state = ParsedPostFileState::Error;
                    file.err = err.to_string();
                });
            }
        }
    }

    pub async fn remove_file<TSender>(
        &self,
        client: &Client<TSender>,
        parsed_file: ArcRwSignal<ParsedPostFile>,
    ) where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_key = self.post_key.get_value();
        if post_key.is_empty() {
            warn!("trying remove files when post wasn't initialized");
            return;
        };

        let (name, state) = parsed_file.with_untracked(|v| (v.name.clone(), v.state.clone()));

        if state != ParsedPostFileState::Uploaded {
            self.remove_file_parsed(&parsed_file);

            return;
        }

        parsed_file.update(|v| {
            v.state = ParsedPostFileState::Removing;
        });

        let result = client
            .post_update_file_remove(post_key, name)
            .send()
            .await
            .into_res()
            .await;

        match result {
            Ok(v) => {
                self.remove_file_parsed(&parsed_file);
            }
            Err(err) => {
                parsed_file.update(|file| {
                    file.state = ParsedPostFileState::Error;
                    file.err = err.to_string();
                });
            }
        }
    }

    fn remove_file_parsed(&self, remove_file: &ArcRwSignal<ParsedPostFile>) {
        let Some(pos) = self
            .files
            .with_untracked(|v| v.iter().position(|v| *v == *remove_file))
        else {
            warn!("trying remove file that doesnt exist");
            return;
        };

        self.files.update(|v| {
            v.remove(pos);
        });

        trace!("remoevd {pos}");
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_upload_state_update() {
    use catsquad_api::{auth::create_auth_cookie_str, utils::rng_str};
    use catsquad_shared::{
        MAX_POST_DESCRIPTION_LENGTH, MAX_POST_TAGS_LENGTH, MAX_POST_TITLE_LENGTH,
    };
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;

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
    assert_eq!(upload.stage.get_untracked(), UploadStateStage::Loading);
    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.title.get_untracked(), "");

    assert_eq!(upload.description_saved.get_untracked().saved, true);
    assert_eq!(upload.description_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.description.get_untracked(), "");

    assert_eq!(upload.tags.get_untracked(), "");

    // try updating when not initialized
    {
        upload.update_title(1, &server.client).await;
        upload.update_description(1, &server.client).await;
        upload.update_tags(1, &server.client).await;

        assert!(upload.err_general.get_untracked().is_empty());
        assert!(upload.err_title.get_untracked().is_empty());
        assert!(upload.err_description.get_untracked().is_empty());
        assert!(upload.err_tags.get_untracked().is_empty());
    }

    upload.init(&server.client).await;

    // asserts that running update without set does nothing
    {
        upload.update_title(1, &server.client).await;
        upload.update_description(1, &server.client).await;
        upload.update_tags(1, &server.client).await;

        assert!(!upload.post_key.get_value().is_empty());
        assert_eq!(upload.stage.get_untracked(), UploadStateStage::Loaded);

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
    }

    // asserts set state change
    {
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
    }

    // asserts update state change
    {
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
    }

    // asserts that invalid value doesnt get pushed

    {
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

    // are errors cleared on success
    {
        upload.set_title(1, "title1");
        upload.set_description(2, "description1");
        upload.set_tags(3, "tags1");

        upload.update_title(7, &server.client).await;
        upload.update_description(8, &server.client).await;
        upload.update_tags(9, &server.client).await;

        assert!(upload.err_title.get_untracked().is_empty());
        assert!(upload.err_description.get_untracked().is_empty());
        assert!(upload.err_tags.get_untracked().is_empty());
    }

    // set state active
    {
        let link = upload.update_state_active(&server.client).await;
        assert!(upload.err_general.get_untracked().is_empty());
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_upload_state_file_add() {
    let (server, _owner, upload) = test_upload_init().await;

    let input_files = vec!["../assets/favicon.ico".to_string()];
    let files_signals = upload.set_files(input_files.clone());

    assert_eq!(files_signals.len(), 1);
    assert_eq!(
        files_signals[0].get_untracked().name,
        "../assets/favicon.ico"
    );

    upload
        .update_file(
            &server.client,
            input_files[0].clone(),
            files_signals[0].clone(),
        )
        .await;

    assert_eq!(files_signals[0].get_untracked().name, "3905551641572326689");
    assert_eq!(files_signals[0].get_untracked().err, "");
    // assert_eq!(upload.er)

    let post_key = upload.post_key.get_value();
    let post1 = server
        .client
        .post_get_by_key(post_key)
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    assert_eq!(post1.file.len(), 1);

    let files_signals = upload.files.get_untracked();
    assert_eq!(files_signals.len(), 1);
    assert_eq!(
        files_signals[0].get_untracked().state,
        ParsedPostFileState::Uploaded
    );
}

#[cfg(test)]
#[tokio::test]
async fn test_upload_state_file_remove() {
    let (server, _owner, upload) = test_upload_init().await;

    let input_files = vec!["../assets/favicon.ico".to_string()];
    let files_signals = upload.set_files(input_files.clone());

    upload
        .update_file(
            &server.client,
            input_files[0].clone(),
            files_signals[0].clone(),
        )
        .await;

    upload
        .remove_file(&server.client, files_signals[0].clone())
        .await;

    let post1 = server
        .client
        .post_get_by_key(upload.post_key.get_value())
        .send()
        .await
        .into_res()
        .await
        .unwrap();

    assert_eq!(post1.file.len(), 0);
}

#[cfg(test)]
async fn test_upload_init() -> (catsquad_api::TestServer, Owner, UploadState) {
    use catsquad_api::auth::create_auth_cookie_str;
    use http::header;

    catsquad_log::init_log();
    let owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;

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
    upload.init(&server.client).await;

    (server, owner, upload)
}
