use anyhow::anyhow;
use catsquad_client::{Client, Response, SchrodingersImage, Sender};
use catsquad_log::prelude::*;
#[cfg(test)]
use catsquad_shared::PostRes;
use catsquad_shared::{PostImage, i64_to_str, str_to_i64};
use leptos::prelude::*;
use std::fmt::{Debug, Display};

use crate::page::create_client;

#[derive(Clone, Copy)]
pub struct PostImagesState {
    // pub err_general: RwSignal<String>,
    pub post_id: Signal<i64>,
    pub images: RwSignal<Vec<ArcRwSignal<ParsedPostImage>>>,
}

#[derive(Clone)]
pub struct ParsedPostImage {
    pub name: String,
    pub hash: i64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub ratio: f64,
    pub uploaded_bytes: u64,
    pub uploaded_percentage: u64,
    pub upload_speed_bytes_a_second: u64,
    pub state: ParsedPostImageState,
    pub err: String,
}

const POST_IMGAGE_ID_PREFIX: &'static str = "img_";

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PostImageId(pub i64);

impl PostImageId {
    pub fn new(hash: i64) -> Self {
        // Self(format!("img_{}", i64_to_str(hash)))
        Self(hash)
    }

    pub fn to_hashtag(&self) -> String {
        format!("#{}", self)
    }

    pub fn to_id(&self) -> String {
        self.to_string()
    }
}

impl<T> From<T> for PostImageId
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let value = value.as_ref();
        let prefix_len = POST_IMGAGE_ID_PREFIX.len();
        trace!("{} <= {}", value.len(), prefix_len);
        if value.len() <= prefix_len {
            return Self(0);
        }
        let first_char = value.chars().next();
        let id = if first_char == Some('#') {
            trace!("{} <= {}", value.len(), prefix_len + 1);
            if value.len() <= prefix_len + 1 {
                return Self(0);
            }
            &value[prefix_len + 1..]
        } else {
            &value[prefix_len..]
        };

        let id = str_to_i64(id);

        Self(id)
    }
}

// impl From<String> for PostImageId {
//     fn from(value: String) -> Self {
//         From::<&str>::from(&value)
//     }
// }

impl Display for PostImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = format!("{}{}", POST_IMGAGE_ID_PREFIX, i64_to_str(self.0));
        write!(f, "{}", output)
    }
}

#[cfg(test)]
#[test]
fn test_post_image_id() {
    init_log();
    let id = PostImageId::new(5);
    let id_str = id.to_string();
    assert_eq!(id_str, "img_5");
    let result = PostImageId::from("img_5");
    assert_eq!(result, id);
    let result = PostImageId::from("#img_5");
    assert_eq!(result, id);
}

impl From<i64> for ParsedPostImage {
    fn from(value: i64) -> Self {
        Self {
            name: value.to_string(),
            hash: value,
            size: 0,
            width: 0,
            height: 0,
            ratio: 0.,
            uploaded_bytes: 0,
            upload_speed_bytes_a_second: 0,
            uploaded_percentage: 0,
            state: ParsedPostImageState::Queue,
            err: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParsedPostImageState {
    Queue,
    Uploading,
    Processing,
    // Idling,
    // Checking,
    Completed,
    Error,
    Removing,
}

impl From<String> for ParsedPostImage {
    fn from(value: String) -> Self {
        Self {
            name: value,
            hash: 0,
            size: 0,
            width: 0,
            height: 0,
            ratio: 0.,
            uploaded_bytes: 0,
            upload_speed_bytes_a_second: 0,
            uploaded_percentage: 0,
            state: ParsedPostImageState::Queue,
            err: String::new(),
        }
    }
}

impl From<&str> for ParsedPostImage {
    fn from(value: &str) -> Self {
        From::<String>::from(value.to_string())
    }
}

impl From<PostImage> for ParsedPostImage {
    fn from(value: PostImage) -> Self {
        Self {
            name: value.hash.to_string(),
            hash: value.hash,
            size: value.size_bytes as u64,
            width: value.width,
            height: value.height,
            ratio: value.width as f64 / value.height as f64,
            uploaded_bytes: 0,
            upload_speed_bytes_a_second: 0,
            uploaded_percentage: 0,
            state: match value.proccesed {
                true => ParsedPostImageState::Completed,
                false => ParsedPostImageState::Processing,
            },
            err: String::new(),
        }
    }
}

impl From<web_sys::File> for ParsedPostImage {
    fn from(file: web_sys::File) -> Self {
        let name = file.name();
        let size = file.size();
        Self {
            name: name,
            hash: 0,
            width: 0,
            height: 0,
            ratio: 0.,
            size: size as u64,
            uploaded_bytes: 0,
            upload_speed_bytes_a_second: 0,
            uploaded_percentage: 0,
            state: ParsedPostImageState::Queue,
            err: String::new(),
        }
    }
}

impl PostImagesState {
    pub fn new(post_id: Signal<i64>, images: RwSignal<Vec<ArcRwSignal<ParsedPostImage>>>) -> Self {
        Self {
            post_id,
            images,
            // files: RwSignal::new(Vec::new()),
            // err_general: RwSignal::new(String::new()),
        }
    }

    pub fn set_images<I>(&self, images: I) -> Vec<ArcRwSignal<ParsedPostImage>>
    where
        I: IntoIterator + Clone,
        I::Item: Into<ParsedPostImage>,
    {
        let images_signals = images
            .into_iter()
            .map(|v| ArcRwSignal::new(Into::<ParsedPostImage>::into(v)))
            .collect::<Vec<ArcRwSignal<ParsedPostImage>>>();

        self.images.update({
            let images_signals = images_signals.clone();
            |parsed_images| {
                parsed_images.extend(images_signals);
            }
        });

        images_signals
    }

    pub async fn update_image<TSender, TImage>(
        &self,
        client: &Client<TSender>,
        source_image: TImage,
        parsed_image: ArcRwSignal<ParsedPostImage>,
    ) where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
        TImage: Into<SchrodingersImage>,
    {
        let post_id = self.post_id.try_get_untracked().unwrap_or_default();
        if post_id == 0 {
            warn!("trying upload images when post wasn't initialized");
            return;
        };

        let result = client
            .post_update_image_add(post_id, vec![source_image])
            .on_progress({
                let image = parsed_image.clone();
                move |stats| {
                    trace!("UPLOADING {stats:?}");
                    image.update(|parsed_image| {
                        parsed_image.state = ParsedPostImageState::Uploading;
                        parsed_image.uploaded_bytes = stats.completed_bytes;
                        parsed_image.upload_speed_bytes_a_second = stats.upload_speed_bytes;
                        parsed_image.uploaded_percentage = stats.completed_precentage;
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
                    parsed_image.update(|image| {
                        image.state = ParsedPostImageState::Error;
                        image.err = "received wrong response".to_string();
                    });
                    return;
                }

                let hash = received_post[0].hash.clone();

                parsed_image.update(|image| {
                    image.name = hash.to_string();
                    image.hash = hash;
                    image.state = ParsedPostImageState::Processing;
                });
            }
            Err(err) => {
                parsed_image.update(|image| {
                    image.state = ParsedPostImageState::Error;
                    image.err = err.to_string();
                });
            }
        }
    }

    pub async fn remove_image<TSender>(
        &self,
        client: &Client<TSender>,
        parsed_image: ArcRwSignal<ParsedPostImage>,
    ) where
        TSender: Sender + Debug + Clone,
        TSender::TResponse: Response + Debug,
    {
        let post_id = self.post_id.try_get_untracked().unwrap_or_default();
        if post_id == 0 {
            warn!("trying remove images when post wasn't initialized");
            return;
        };

        let (hash, state) = parsed_image.with_untracked(|v| (v.hash, v.state.clone()));

        if state == ParsedPostImageState::Queue {
            self.remove_image_parsed(&parsed_image);
            return;
        }

        // TODO add uploading cancel button

        parsed_image.update(|v| {
            v.state = ParsedPostImageState::Removing;
        });

        let result = client
            .post_update_image_remove(post_id, hash)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {
                self.remove_image_parsed(&parsed_image);
            }
            Err(err) => {
                parsed_image.update(|image| {
                    image.state = ParsedPostImageState::Error;
                    image.err = format!("err removing image: {err}");
                });
            }
        }
    }

    fn remove_image_parsed(&self, remove_image: &ArcRwSignal<ParsedPostImage>) {
        let Some(pos) = self
            .images
            .with_untracked(|v| v.iter().position(|v| *v == *remove_image))
        else {
            warn!("trying remove image that doesnt exist");
            return;
        };

        self.images.update(|v| {
            v.remove(pos);
        });

        trace!("remoevd {pos}");
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_post_images_state_image_add() {
    let (server, _owner, upload, post1) = test_upload_init("test_upload_state_images_add").await;

    let input_images = vec!["../assets/favicon.ico".to_string()];
    let images_signals = upload.set_images(input_images.clone());

    assert_eq!(images_signals.len(), 1);
    assert_eq!(
        images_signals[0].with_untracked(|v| v.state),
        ParsedPostImageState::Queue
    );
    assert_eq!(
        images_signals[0].get_untracked().name,
        "../assets/favicon.ico"
    );

    // upload.fil

    upload
        .update_image(
            &server.client,
            input_images[0].clone(),
            images_signals[0].clone(),
        )
        .await;

    assert_eq!(
        images_signals[0].get_untracked().name,
        "3905551641572326689"
    );
    assert_eq!(images_signals[0].get_untracked().err, "");
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

    assert_eq!(post1.images_hashes.len(), 1);

    let images_signals = upload.images.get_untracked();
    assert_eq!(images_signals.len(), 1);
    assert_eq!(
        images_signals[0].get_untracked().state,
        ParsedPostImageState::Processing
    );
}

#[cfg(test)]
#[tokio::test]
async fn test_upload_state_image_remove() {
    let (server, _owner, upload, post1) = test_upload_init("test_upload_state_image_remove").await;

    let input_images = vec!["../assets/favicon.ico".to_string()];
    let images_signals = upload.set_images(input_images.clone());
    let image = input_images[0].clone();
    let image_signal = images_signals[0].clone();

    {
        let images = upload.images.get();
        assert_eq!(images.len(), 1);
        assert_eq!(
            images[0].with_untracked(|v| v.state),
            ParsedPostImageState::Queue
        );

        upload
            .remove_image(&server.client, image_signal.clone())
            .await;
        let image_err = image_signal.with_untracked(|v| v.err.clone());

        let images = upload.images.get_untracked();
        assert_eq!(image_err, "");
        assert_eq!(images.len(), 0);
    }

    upload
        .update_image(&server.client, image, image_signal.clone())
        .await;

    upload
        .remove_image(&server.client, image_signal.clone())
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

    assert_eq!(post1.images_hashes.len(), 0);
}

#[cfg(test)]
async fn test_upload_init(
    db_name: &str,
) -> (catsquad_api::TestServer, Owner, PostImagesState, PostRes) {
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
    let upload = PostImagesState::new(post1.id.into(), RwSignal::new(Vec::new()));
    // upload.init(&server.client).await;

    (server, owner, upload, post1)
}
