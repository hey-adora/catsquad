use std::{fmt::Debug, time::Duration};

use crate::{AutoTextArea, Errs, Nav, SVGTrash, hook::Spawner, page::create_client};
use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{
    MAX_POST_DESCRIPTION_LENGTH, MAX_POST_TAGS_LENGTH, MAX_POST_TITLE_LENGTH, PostAddErr,
    validate_post_description, validate_post_tags, validate_post_title,
};
use catsquad_web_utils::{
    file::GetFiles,
    interval,
    prelude::rem_to_px,
    time::{ns_to_str, time_now_ns},
};
use leptos::prelude::*;
use web_sys::{File, HtmlInputElement, HtmlTextAreaElement, MouseEvent};

const AUTO_SAVE_TIME: u128 = 3000000000; // 3s

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
    use catsquad_shared::{MAX_POST_DESCRIPTION_LENGTH, MAX_POST_TAGS_LENGTH};
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

#[component]
pub fn Upload() -> impl IntoView {
    let time = time_now_ns();
    let spawner = Spawner::new();
    let upload = UploadState::new(time);

    Effect::new(move || {
        spawner.spawn(async move {
            let client = create_client();
            upload.init(&client).await;
        });
    });

    view! {
        <Nav/>
        <div class="flex flex-col gap-4 max-w-[25rem] mx-auto" >
            <TitleEdit upload/>
            <ImagesEdit upload/>
            <DescriptionEdit upload/>
            <TagsEdit upload/>
        </div>
    }
}

// trace!(
//     "auto save title - checking - {} - {} >= {} = elapsed({}) && saved({})",
//     time, meta_data.set_at, AUTO_SAVE_TIME, elapsed, meta_data.saved
// );

#[derive(Clone)]
pub struct ParsedFile {
    pub file: File,
    pub name: String,
}

#[component]
pub fn ImagesEdit(upload: UploadState) -> impl IntoView {
    let input_files = NodeRef::new();
    let files = RwSignal::<Vec<ParsedFile>>::new(Vec::new());
    let on_file_change = move |e| {
        let Some(new_files) = (input_files.get_untracked() as Option<HtmlInputElement>)
            .and_then(|f: HtmlInputElement| f.files())
            .map(|f| f.get_files())
        else {
            return;
        };
        files.update(|v| {
            v.extend(new_files.into_iter().map(|file| {
                let name = file.name();

                ParsedFile { file, name }
            }))
        });

        //
    };
    let view_files = move || {
        files
            .get()
            .into_iter()
            .map(|file| {
                let name = file.name.clone();
                view! {
                    <PreviewFile
                        name=move|| name.clone()
                    />
                }
            })
            .collect_view()
            .into_any()
    };
    let is_valid = move || ValidState::Empty;
    view! {
        <div class="flex flex-col gap-2">
            <p class="text-[1.3rem] text-base0F ">"Images"</p>
            <Errs error=move||upload.err_general.get() />
            <EditArea
                class=move||"flex gap-2"
                required=false
                is_valid=move||is_valid()
                >
                { view_files }
                <PreviewAdd fn_for=move||"image"/>
                <input class="absolute z-[-1] opacity-0" on:change=on_file_change type="file" id="image" name="image" node_ref=input_files multiple />
            </EditArea>
        </div>
    }
}

#[component]
pub fn PreviewFile(
    #[prop(optional, into)] name: Option<Callback<(), String>>,
    // #[prop(optional, into)] class: Option<Callback<(), String>>,
    // #[prop(optional, into)] hash: Option<Callback<(), String>>,
    // #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
    let fn_name = move || name.map(|v| v.run(())).unwrap_or_default();

    // let location = use_location();

    view! { <div
            id="previw_add"
            // for=move||fn_for()
            class=move ||  {
                // let hash = location.hash.get();
                // trace!("hash: {hash}");
                format!(" grid place-items-center h-[5rem] w-[5rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05")
            }
            // style:background-image=move || format!("url(\"{url}\")")
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { fn_name }
              </p>
              <p class="text-[0.7rem]">
                  "2.78 MB/s"
              </p>
              <p class="text-[0.9rem]">
                  "45%"
              </p>
              <p class="text-[0.7rem]">
                  "248MB/1GB"
              </p>
              <SVGTrash class="size-[1.1rem]" />
            </div>
    }
}

#[component]
pub fn PreviewAdd(
    #[prop(optional, into)] fn_for: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] hash: Option<Callback<(), String>>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
    let fn_for = move || fn_for.map(|v| v.run(())).unwrap_or_default();

    // let location = use_location();

    view! { <label
            id="previw_add"
            for=move||fn_for()
            class=move ||  {
                // let hash = location.hash.get();
                // trace!("hash: {hash}");
                format!("text-[2rem] grid place-items-center h-[5rem] w-[5rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05")
            }
            // style:background-image=move || format!("url(\"{url}\")")
            >"+"</label>
    }
}

#[component]
pub fn TitleEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
    let on_update = move || {
        spawner.spawn(async move {
            let time = time_now_ns();
            let client = create_client();
            trace!("auto save title - running");
            upload.update_title(time, &client).await;
        });
    };
    let on_set = move |value| {
        let time = time_now_ns();
        upload.set_title(time, value);
    };
    view! {
        <TextEditArea
            on_update
            on_set
            max_length=MAX_POST_TITLE_LENGTH
            min_height_rem=2
            title="Title"
            saved_metadata=upload.title_saved
            input_text=upload.title
            errors=upload.err_title/>
    }
}

#[component]
pub fn DescriptionEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
    let on_update = move || {
        spawner.spawn(async move {
            let time = time_now_ns();
            let client = create_client();
            trace!("auto save description - running");
            upload.update_description(time, &client).await;
        });
    };
    let on_set = move |value| {
        let time = time_now_ns();
        upload.set_description(time, value);
    };
    view! {
        <TextEditArea
            on_update
            on_set
            max_length=MAX_POST_DESCRIPTION_LENGTH
            min_height_rem=10
            title="Description"
            saved_metadata=upload.description_saved
            input_text=upload.description
            errors=upload.err_description/>
    }
}

#[component]
pub fn TagsEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
    let on_update = move || {
        spawner.spawn(async move {
            let time = time_now_ns();
            let client = create_client();
            trace!("auto save tags - running");
            upload.update_tags(time, &client).await;
        });
    };
    let on_set = move |value| {
        let time = time_now_ns();
        upload.set_tags(time, value);
    };
    view! {
        <TextEditArea
            on_update
            on_set
            max_length=MAX_POST_TAGS_LENGTH
            min_height_rem=8
            title="Tags"
            saved_metadata=upload.tags_saved
            input_text=upload.tags
            errors=upload.err_tags/>
    }
}

#[derive(Default, Clone, Copy, strum::EnumIs)]
pub enum ValidState {
    Empty,
    #[default]
    Error,
    Valid,
}

#[component]
pub fn TextEditArea(
    #[prop(optional, into)] title: String,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] on_set: Option<Callback<String>>,
    #[prop(optional, into)] on_update: Option<Callback<()>>,
    #[prop(default = 1)] min_height_rem: u64,
    #[prop(default = 100)] max_length: usize,
    saved_metadata: RwSignal<FieldSaved>,
    input_text: RwSignal<String>,
    errors: RwSignal<String>,
) -> impl IntoView {
    let title_clone1 = title.clone();
    let title_clone2 = title.clone();

    let _handle = interval::new(
        move || {
            let time = time_now_ns();
            let Some(meta_data) = saved_metadata.try_get_untracked() else {
                return;
            };
            let elapsed = time.saturating_sub(meta_data.set_at) >= AUTO_SAVE_TIME;
            let result = saved_metadata.try_update(|v| v.checked_at = time);
            if result.is_none() {
                return;
            }
            if elapsed && !meta_data.saved {
                if let Some(f) = on_update {
                    f.run(());
                }
            }
        },
        Duration::from_secs(1),
    );

    let is_valid = move || -> ValidState {
        if input_text.with(|v| v.is_empty()) {
            return ValidState::Empty;
        }

        if errors.with(|v| !v.is_empty()) {
            return ValidState::Error;
        }

        ValidState::Valid
    };

    let fn_on_input = move |e: HtmlTextAreaElement| {
        let value = e.value();
        if let Some(f) = on_set {
            f.run(value);
        }
    };

    let fn_on_focusout = move |e: HtmlTextAreaElement| {
        if let Some(f) = on_update {
            f.run(());
        }
    };

    let fn_track = move || {
        input_text.track();
    };

    let required_text_color = move || match is_valid() {
        ValidState::Valid => "text-base0B",
        ValidState::Error => "text-base08",
        ValidState::Empty => match required {
            true => "text-base08",
            false => "text-base0A",
        },
    };

    let required_text = move || match is_valid() {
        ValidState::Valid => "is valid",
        ValidState::Error => "is invalid",
        ValidState::Empty => match required {
            true => "is required",
            false => "is optional",
        },
    };

    let saved_text = move || {
        if !errors.with(|v| v.is_empty()) {
            return "invalid".to_string();
        }
        let data = saved_metadata.get();
        // trace!("saved text {} {data:?}", title_clone1);
        match data.saved {
            true => "saved.".to_string(),
            false => {
                let time = time_now_ns();
                let saving_in = AUTO_SAVE_TIME.saturating_sub(time.saturating_sub(data.set_at));
                let saving_in = ns_to_str(saving_in);
                format!("saving in {}", saving_in)
            }
        }
    };

    let saved_text_color = move || {
        if !errors.with(|v| v.is_empty()) {
            return "text-base08";
        }
        let saved = saved_metadata.with(|v| v.saved);
        match saved {
            true => "text-base0B",
            false => "text-base05",
        }
    };

    view! {
        <div class="flex flex-col ">
            <div class="flex gap-2 flex-wrap place-items-center">
                <p class="text-[1.3rem] text-base0F ">{ title_clone1 }</p>
                <ul>
                    <li class=move|| format!("ml-[1rem] list-disc {}", required_text_color()) >{required_text}</li>
                </ul>
            </div>
            <Errs class=move||"mb-2" error=move||errors.get()/>


            <EditArea
                required
                is_valid=move||is_valid()
                class=move||"flex flex-col gap-2"
                >
                <AutoTextArea
                    class=move||"w-full select-none"
                    placeholder=move||title.to_lowercase()
                    track=fn_track
                    on_focusout=fn_on_focusout
                    on_input=fn_on_input
                    min_height=rem_to_px(min_height_rem).unwrap_or_default()>
                    { move || input_text.get() }
                </AutoTextArea>
                <div class="flex justify-between ">
                    <LengthCounter
                        id=move||format!("{}_counter", title_clone2)
                        counter_current=move||input_text.with(|v|v.trim().len())
                        counter_max=move||max_length
                        />
                    <div class=move||format!("{}", saved_text_color())>{saved_text}</div>
                </div>

            </EditArea>

        </div>
    }
}

#[component]
pub fn EditArea(
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] is_valid: Option<Callback<(), ValidState>>,
    #[prop(optional)] required: bool,
    children: Children,
) -> impl IntoView {
    let is_valid = move || is_valid.map(|v| v.run(())).unwrap_or_default();
    let fn_class = move || class.map(|v| v.run(())).unwrap_or_default();

    let container_color = move || match is_valid() {
        ValidState::Valid => "border-base0B bg-base0B/5",
        ValidState::Error => "border-base08 bg-base08/5",
        ValidState::Empty => match required {
            true => "border-base08 bg-base08/5",
            false => "border-base0A bg-base0A/5",
        },
    };

    view! {
        <div class=move || format!("border-2 rounded-lg px-3 py-2 {} {}", container_color(), fn_class() )>
            {children()}
        </div>
    }
}

#[component]
pub fn LengthCounter(
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] counter_current: Option<Callback<(), usize>>,
    #[prop(optional, into)] counter_max: Option<Callback<(), usize>>,
) -> impl IntoView {
    let counter_current_fn = move || counter_current.map(|v| v.run(())).unwrap_or_default();
    let counter_max_fn = move || counter_max.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();

    view! {
        <div class=move || format!("{} {}", if counter_current_fn() > counter_max_fn() {"text-base08"} else {""}, class_fn())>
            <span id=move||id_fn()>{counter_current_fn}</span>"/"{counter_max_fn}
        </div>
    }
}
