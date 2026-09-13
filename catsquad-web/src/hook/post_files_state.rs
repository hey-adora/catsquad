use catsquad_client::{Client, Response, SchrodingersFile, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::PostFile;
#[cfg(test)]
use catsquad_shared::PostRes;
use leptos::prelude::*;
use std::fmt::Debug;

#[derive(Clone, Copy)]
pub struct PostFilesState {
    // pub err_general: RwSignal<String>,
    pub post_id: Signal<i64>,
    pub files: RwSignal<Vec<ArcRwSignal<ParsedPostFile>>>,
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

impl From<i64> for ParsedPostFile {
    fn from(value: i64) -> Self {
        Self {
            name: value.to_string(),
            size: 0,
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
            name: value.hash.to_string(),
            size: value.size_bytes as u64,
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

impl PostFilesState {
    pub fn new(post_id: Signal<i64>, files: RwSignal<Vec<ArcRwSignal<ParsedPostFile>>>) -> Self {
        Self {
            post_id,
            files,
            // files: RwSignal::new(Vec::new()),
            // err_general: RwSignal::new(String::new()),
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
        let post_id = self.post_id.try_get_untracked().unwrap_or_default();
        if post_id == 0 {
            warn!("trying upload files when post wasn't initialized");
            return;
        };

        let result = client
            .post_update_file_add(post_id, vec![source_file])
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
            .into_json()
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
                    file.name = hash.to_string();
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
        let post_id = self.post_id.try_get_untracked().unwrap_or_default();
        if post_id == 0 {
            warn!("trying remove files when post wasn't initialized");
            return;
        };

        let (name, state) = parsed_file.with_untracked(|v| {
            (
                i64::from_str_radix(&v.name, 10).unwrap_or_default(),
                v.state.clone(),
            )
        });

        if state != ParsedPostFileState::Uploaded {
            self.remove_file_parsed(&parsed_file);

            return;
        }

        parsed_file.update(|v| {
            v.state = ParsedPostFileState::Removing;
        });

        let result = client
            .post_update_file_remove(post_id, name)
            .send()
            .await
            .into_json()
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
async fn test_post_files_state_file_add() {
    let (server, _owner, upload, post1) = test_upload_init("test_upload_state_file_add").await;

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

    // post_add in this context only GETS the post
    // post_add doesn't create new draft if one already exists
    let post1 = server
        .client
        .post_add("", "", "")
        .send()
        .await
        .into_json()
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
    let (server, _owner, upload, post1) = test_upload_init("test_upload_state_file_remove").await;

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

    // post_add in this context only GETS the post
    // post_add doesn't create new draft if one already exists
    let post1 = server
        .client
        .post_add("", "", "")
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    assert_eq!(post1.file.len(), 0);
}

#[cfg(test)]
async fn test_upload_init(
    db_name: &str,
) -> (catsquad_api::TestServer, Owner, PostFilesState, PostRes) {
    use catsquad_api::auth::create_auth_cookie_str;
    use catsquad_shared::uuid_to_str;
    use http::header;

    catsquad_log::init_log();
    let owner = crate::init_owner();
    let server = catsquad_api::TestServer::new(0, db_name).await;

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

    let post1 = server
        .client
        .post_add("", "", "")
        .send()
        .await
        .into_json()
        .await
        .unwrap();

    // upload.init creates new post draft
    let upload = PostFilesState::new(post1.id.into());
    // upload.init(&server.client).await;

    (server, owner, upload, post1)
}
