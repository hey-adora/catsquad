use std::{fmt::Debug, time::Duration};

use crate::{AutoTextArea, Errs, Nav, hook::Spawner, page::create_client};
use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_shared::{MAX_POST_TITLE_LENGTH, PostAddErr, validate_post_title};
use catsquad_web_utils::{
    interval,
    prelude::rem_to_px,
    time::{ns_to_str, time_now_ns},
};
use leptos::prelude::*;
use web_sys::HtmlTextAreaElement;

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
    pub tags: RwSignal<String>,
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
            tags: RwSignal::new(String::new()),
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
                    v.set_at = time;
                });
                self.err_title.update(|v| v.clear());
            }
            Err(err) => self.err_title.set(err),
        }
        self.title.set(title.to_string());
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
}

#[cfg(test)]
#[tokio::test]
async fn test_upload_state() {
    use catsquad_api::{auth::create_auth_cookie_str, utils::rng_str};
    use http::header;

    catsquad_log::init_log();
    let _owner = crate::init_owner();
    let server = catsquad_api::TestServer::new().await;
    server.state.set_time(0).await;

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
    assert_eq!(upload.description.get_untracked(), "");
    assert_eq!(upload.tags.get_untracked(), "");

    upload.init(&server.client).await;
    server.state.set_time(1).await;
    upload.update_title(1, &server.client).await;

    assert!(!upload.post_key.get_value().is_empty());
    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.title.get_untracked(), "");
    assert_eq!(upload.description.get_untracked(), "");
    assert_eq!(upload.tags.get_untracked(), "");

    upload.set_title(1, "title1");

    assert_eq!(upload.title_saved.get_untracked().saved, false);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 0);
    assert_eq!(upload.title_saved.get_untracked().set_at, 1);
    assert_eq!(upload.title.get_untracked(), "title1");
    assert_eq!(upload.description.get_untracked(), "");
    assert_eq!(upload.tags.get_untracked(), "");

    upload.update_title(1, &server.client).await;

    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 1);
    assert_eq!(upload.title_saved.get_untracked().set_at, 1);
    assert_eq!(upload.title.get_untracked(), "title1");
    assert_eq!(upload.description.get_untracked(), "");
    assert_eq!(upload.tags.get_untracked(), "");
    assert_eq!(upload.err_title.get_untracked(), "");

    let title = rng_str(MAX_POST_TITLE_LENGTH + 1);
    upload.set_title(2, title);

    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 1);
    assert_eq!(upload.title_saved.get_untracked().set_at, 1);

    upload.update_title(2, &server.client).await;

    assert_eq!(upload.title_saved.get_untracked().saved, true);
    assert_eq!(upload.title_saved.get_untracked().saved_at, 1);
    assert_eq!(upload.title_saved.get_untracked().set_at, 1);
    assert_ne!(upload.title.get_untracked(), "title1");
    assert_eq!(upload.description.get_untracked(), "");
    assert_eq!(upload.tags.get_untracked(), "");
    assert!(!upload.err_title.get_untracked().is_empty());
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
        <div class="flex flex-col max-w-[25rem] mx-auto" >
            <TitleEdit upload/>
        </div>
    }
}

// trace!(
//     "auto save title - checking - {} - {} >= {} = elapsed({}) && saved({})",
//     time, meta_data.set_at, AUTO_SAVE_TIME, elapsed, meta_data.saved
// );
#[component]
pub fn TitleEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
    let _handle = interval::new(
        move || {
            let time = time_now_ns();
            let Some(meta_data) = upload.title_saved.try_get_untracked() else {
                return;
            };
            let elapsed = time.saturating_sub(meta_data.set_at) >= AUTO_SAVE_TIME;
            let result = upload.title_saved.try_update(|v| v.checked_at = time);
            if result.is_none() {
                return;
            }
            if elapsed && !meta_data.saved {
                let client = create_client();
                spawner.spawn(async move {
                    trace!("auto save title - running");
                    upload.update_title(time, &client).await;
                });
            }
        },
        Duration::from_secs(1),
    );

    let on_input = move |v: String| {
        let time = time_now_ns();
        upload.set_title(time, v);
    };

    let on_focusout = move |v: String| {
        let time = time_now_ns();
        let client = create_client();
        spawner.spawn(async move {
            trace!("auto save title - running");
            upload.update_title(time, &client).await;
        });
    };

    view! {
        <TextEditArea
            on_input
            on_focusout
            title="Title"
            saved_metadata=upload.title_saved
            input_text=upload.title
            errors=upload.err_title/>
    }
}

#[component]
pub fn TextEditArea(
    #[prop(optional, into)] title: String,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] on_input: Option<Callback<String>>,
    #[prop(optional, into)] on_focusout: Option<Callback<String>>,
    saved_metadata: RwSignal<FieldSaved>,
    input_text: RwSignal<String>,
    errors: RwSignal<String>,
) -> impl IntoView {
    #[derive(Clone, Copy, strum::EnumIs)]
    enum ValidState {
        Empty,
        Error,
        Valid,
    }

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
        if let Some(f) = on_input {
            f.run(value);
        }
    };

    let fn_on_focusout = move |e: HtmlTextAreaElement| {
        let value = e.value();
        if let Some(f) = on_focusout {
            f.run(value);
        }
    };

    let fn_track = move || {
        input_text.track();
    };

    let container_color = move || match is_valid() {
        ValidState::Valid => "border-base0B bg-base0B/5",
        ValidState::Error => "border-base08 bg-base08/5",
        ValidState::Empty => match required {
            true => "border-base08 bg-base08/5",
            false => "border-base0A bg-base0A/5",
        },
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
                <h1 class="text-[1.3rem] text-base0F ">"Title"</h1>
                <ul>
                    <li class=move|| format!("ml-[1rem] list-disc {}", required_text_color()) >{required_text}</li>
                </ul>
            </div>
            <Errs class=move||"mb-2" error=move||errors.get()/>
            <div class=move || format!("flex flex-col gap-2 border-2  rounded-lg px-3 py-2 {} ", container_color())>
                <AutoTextArea
                    class=move||"w-full select-none"
                    placeholder=move||title.to_lowercase()
                    track=fn_track
                    on_focusout=fn_on_focusout
                    on_input=fn_on_input
                    min_height=rem_to_px(1).unwrap_or_default()>
                    { move || input_text.get() }
                </AutoTextArea>
                <div class="flex justify-between ">
                    <LengthCounter
                        counter_current=move||input_text.with(|v|v.trim().len())
                        counter_max=move||MAX_POST_TITLE_LENGTH
                        />
                    <div class=move||format!("{}", saved_text_color())>{saved_text}</div>
                </div>
            </div>
        </div>
    }
}

// #[prop(optional, into)] on_input: Option<Callback<HtmlTextAreaElement>>,
#[component]
pub fn LengthCounter(
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] counter_current: Option<Callback<(), usize>>,
    #[prop(optional, into)] counter_max: Option<Callback<(), usize>>,
) -> impl IntoView {
    let counter_current_fn = move || counter_current.map(|v| v.run(())).unwrap_or_default();
    let counter_max_fn = move || counter_max.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();

    view! {
        <div class=move || format!("{} {}", if counter_current_fn() > counter_max_fn() {"text-base08"} else {""}, class_fn())>
            <span id="description_length">{counter_current_fn}</span>"/"{counter_max_fn}
        </div>
    }
}
