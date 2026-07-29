use std::fmt::Debug;

use crate::{AutoTextArea, Nav, hook::Spawner, page::create_client};
use catsquad_client::{Client, Response, Sender};
use catsquad_log::prelude::*;
use catsquad_web_utils::prelude::rem_to_px;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct UploadState {
    pub title: RwSignal<String>,
    pub description: RwSignal<String>,
    pub tags: RwSignal<String>,
}

impl UploadState {
    pub fn new() -> Self {
        Self {
            title: RwSignal::new(String::new()),
            description: RwSignal::new(String::new()),
            tags: RwSignal::new(String::new()),
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
            Err(err) => {
                //
            }
        }
    }
}

#[component]
pub fn Upload() -> impl IntoView {
    let spawner = Spawner::new();
    let upload = UploadState::new();

    Effect::new(move || {
        spawner.spawn(async move {
            let client = create_client();
            upload.init(&client).await;
        });
        // upload.await.init(client)
    });

    view! {
        <Nav/>
        <div class="flex flex-col max-w-[25rem] mx-auto" >
            <div>
                <h1 class="text-[1.3rem] text-base0F">"Title"</h1>
                <div class="flex flex-col gap-2 border-2 border-base08 rounded-lg px-3 py-2 bg-base08/5 ">
                    <AutoTextArea
                        class=move||"w-full select-none"
                        min_height=rem_to_px(1).unwrap_or_default()>
                        { move || upload.title.get() }
                    </AutoTextArea>
                    <div class="flex justify-between ">
                        <div>"0/100"</div>
                        <div class="">"saving..."</div>
                    </div>
                </div>
            </div>


        </div>
    }
}
