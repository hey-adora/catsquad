use std::{sync::Arc, time::Duration};

use super::component_edit_area::EditArea;
use super::upload_state::UploadState;
use crate::{
    Errs, Floater, PageState, SVGTrash, Zone, ZoneData,
    hook::{EventListener, ParsedPostImage, ParsedPostImageState, PostImagesState, Spawner},
    page::{create_client, upload::component_edit_text::ValidState},
};
use catsquad_log::prelude::*;
use catsquad_shared::link_relative_post_thumbnail_bytes_get_by_hash;
use catsquad_web_utils::prelude::*;
use leptos::{ev, html::div, prelude::*, task::spawn_local};
use wasm_bindgen::JsCast;
use web_sys::{File, HtmlElement, HtmlInputElement, MouseEvent};

// TODO
// add cancel upload
// add proccesed interval check
// fix styling

#[component]
pub fn ImagesEdit(
    #[prop(into)] author_username: Signal<String>,
    #[prop(optional, into)] post_id: Signal<i64>,
    #[prop(optional, into)] images: RwSignal<Vec<ArcRwSignal<ParsedPostImage>>>,
) -> impl IntoView {
    // let on_link = move |f| format!("");
    let is_valid = move || {
        let state = images.with(|v| {
            if v.is_empty() {
                return ValidState::Empty;
            }
            let has_err = v
                .iter()
                .any(|v| v.with(|v| v.state == ParsedPostImageState::Error));
            if has_err {
                return ValidState::Error;
            }
            ValidState::Valid
        });
        state
    };
    view! {
        <div class="flex flex-col gap-2">
            <p class="text-[1.3rem] text-base0F  ">"Images"</p>
            <EditArea
                class=move||"flex flex-wrap gap-4 "
                required=false
                is_valid=move||is_valid()
                >
                <ImagesView author_username post_id images/>
            </EditArea>
        </div>
    }
}

#[component]
pub fn ImagesView(
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] on_link: Option<Callback<ArcRwSignal<ParsedPostImage>, String>>,
    #[prop(into)] author_username: Signal<String>,
    #[prop(optional, into)] post_id: Signal<i64>,
    #[prop(optional, into)] images: RwSignal<Vec<ArcRwSignal<ParsedPostImage>>>,
) -> impl IntoView {
    let spawner = Spawner::new();

    let page = PageState::get();
    let input_files = NodeRef::new();
    let post_files_state = PostImagesState::new(post_id, images);
    let images = post_files_state.images;

    let on_click = move |e: MouseEvent| {
        if let Some(f) = on_click {
            f.run(e)
        }
    };

    let on_link = move |file: ArcRwSignal<ParsedPostImage>| {
        if let Some(f) = on_link {
            f.run(file)
        } else {
            "".to_string()
        }
    };

    let on_file_change = move |e| {
        trace!("on_file_change");
        let Some(new_files) = (input_files.get_untracked() as Option<HtmlInputElement>)
            .and_then(|f: HtmlInputElement| f.files())
            .map(|f| f.get_files())
        else {
            warn!("upload canceled, failed to get files");
            return;
        };

        let parset_files = post_files_state.set_images(new_files.clone());
        for (parset_file, new_file) in parset_files.into_iter().zip(new_files) {
            spawn_local(async move {
                let client = create_client();
                post_files_state
                    .update_image(&client, new_file, parset_file)
                    .await;
            });
        }
    };

    let zones = ZoneData::new();
    let on_drop = move |(zone_index, float_index)| {
        trace!("on_drop {} {}", zone_index, float_index);
    };

    let view_files = move || {
        trace!("view_files updating");
        // upload.
        images
            .get()
            .into_iter()
            .enumerate()
            .map(|(index, file)| {
                let block = match file.with(|v| v.state.clone()) {
                ParsedPostImageState::Queue => view! {
                    <FileQueuePreview
                        file
                    />
                }
                .into_any(),
                ParsedPostImageState::Removing => view! {
                    <FileQueuePreview
                        file
                    />
                }
                .into_any(),
                ParsedPostImageState::Uploading => view! {
                    <FileUploadingPreview
                        file
                    />
                }
                .into_any(),

                ParsedPostImageState::Processing => view! {
                    <FileProcessingPreview
                        author_username
                        post_files_state
                        file
                    />
                }
                .into_any(),
                ParsedPostImageState::Completed
                // | ParsedPostFileState::Idling
                // | ParsedPostFileState::Checking
                 => view! {
                    <FileCompletedPreview
                        on_link
                        on_click
                        zones
                        file_index=index
                        author_username
                        post_files_state
                        file
                    />
                }
                .into_any(),
                ParsedPostImageState::Error => view! {
                    <FileErrorPreview
                        author_username
                        post_files_state
                        file
                    />
                }
                .into_any(),
                };
                view! {
                    <Zone zone_index=index zones on_drop/>
                    {block}
                }
            })
            .collect_view()
            .into_any()
    };

    let when_is_owner = move || page.acc_username() == author_username.get();
    // let last_index = move || images.with(|v| v.len());

    view! {
        { view_files }
        <Zone zone_index=1000 zones on_drop/>
        <Show when=when_is_owner>
            <PreviewAdd fn_for=move||"image"/>
            <input class="absolute z-[-1] opacity-0" on:change=on_file_change type="file" id="image" name="image" node_ref=input_files multiple />
        </Show>
    }
}

#[derive(Clone)]
struct ProccessingState {
    // pub post_id: Signal<i64>,
    pub check_state: Signal<CheckState>,
    pub file: ArcRwSignal<ParsedPostImage>,
}

#[derive(Clone, Copy, Default)]
struct CheckState {
    // pub stage: ProccessingLoadingStage,
    pub attemps: usize,
    pub attempted_at: u64,
}

// enum ProccessingLoadingStage {
//     Idle,
//     Busy,
//     Finished,
// }

impl ProccessingState {
    pub fn new(file: ArcRwSignal<ParsedPostImage>) -> Self {
        Self {
            file,
            check_state: CheckState::default().into(),
        }
    }
    pub async fn check(self, time: u64) {
        // self.check_state.with_untracked(|v| v.stage == ProccessingLoadingStage::Busy);
        // let post_id = self.post_id.get_untracked();

        let file_hash = self.file.with_untracked(|v| v.hash);
        trace!("checking if file {file_hash} is proccesed");

        let file = self.file;
        let client = create_client();

        let result = client
            .post_file_status_get_by_hash(file_hash)
            .send()
            .await
            .into_json()
            .await;

        match result {
            Ok(v) => {
                if v.is_proccesed {
                    file.update(|v| v.state = ParsedPostImageState::Completed);
                }
            }
            Err(err) => {
                error!("{err}");
            }
        }
    }
}

#[component]
pub fn FileQueuePreview(file: ArcRwSignal<ParsedPostImage>) -> impl IntoView {
    let name = {
        let file = file.clone();
        move || file.with(|v| v.name.clone())
    };

    // let size = bytes_to_str(file.size as u64);

    view! { <div
            class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { name }
              </p>
              <p class="text-[0.7rem]">
                  "preparing..."
              </p>
            </div>
    }
}

#[component]
pub fn FileUploadingPreview(file: ArcRwSignal<ParsedPostImage>) -> impl IntoView {
    let view_speed_text = {
        let file = file.clone();
        move || file.with(|v| format!("{}/s", bytes_to_str(v.upload_speed_bytes_a_second)))
    };
    let view_name_text = {
        let file = file.clone();
        move || file.with(|v| v.name.clone())
    };
    let view_percent_width = {
        let file = file.clone();
        move || file.with(|v| format!("{}%", v.uploaded_percentage))
    };
    let view_percent_text = view_percent_width.clone();
    let view_size_text = {
        move || {
            file.with(|v| {
                format!(
                    "{}/{}",
                    bytes_to_str(v.uploaded_bytes),
                    bytes_to_str(v.size)
                )
            })
        }
    };

    view! { <div
            class="p-2 relative grid grid-rows-[auto_1fr_auto] gap-1 place-items-center size-[8rem] rounded-xl bg-base02 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { view_name_text }
              </p>
              <div class="w-full text-center">
                  <p class="text-[0.7rem]">
                      {view_speed_text}
                  </p>
                  <div class="h-[1.5rem] overflow-hidden  text-base05 font-bold place-items-center text-[0.9rem] bg-base01 w-full rounded-full relative ">
                      <p class="absolute left-0 top-0 w-full h-full grid place-items-center">{view_percent_text}</p>
                      <p
                          class="h-full bg-base03 mr-auto"
                          style:width=view_percent_width
                          >
                      </p>
                  </div>
              </div>
              <p class="text-[0.7rem]">
                  {view_size_text}
              </p>
            </div>
    }
}

#[component]
pub fn FileCompletedPreview(
    #[prop(into)] zones: ZoneData,
    #[prop(into)] file_index: Signal<usize>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] on_link: Option<Callback<ArcRwSignal<ParsedPostImage>, String>>,
    #[prop(into)] author_username: Signal<String>,
    post_files_state: PostImagesState,
    file: ArcRwSignal<ParsedPostImage>,
) -> impl IntoView {
    // let name = {
    //     let file = file.clone();
    //     move || file.with(|v| v.name.clone())
    // };

    let link = {
        let file = file.clone();
        move || {
            if let Some(f) = on_link {
                f.run(file.clone())
            } else {
                String::new()
            }
        }
    };

    let on_click = move |e: MouseEvent| {
        if let Some(f) = on_click {
            f.run(e)
        }
    };

    let style_bg_img = {
        let file = file.clone();
        move || {
            let post_id = post_files_state.post_id.get();
            let file_hash = file.with(|v| v.hash);
            format!(
                "url(\"{}\")",
                link_relative_post_thumbnail_bytes_get_by_hash(post_id, file_hash)
            )
        }
    };

    let when_is_link = {
        let link = link.clone();
        move || !link().is_empty()
    };

    let fallback = {
        let file = file.clone();
        let style_bg_img = style_bg_img.clone();
        move || {
            view! {
                <div
                on:click=on_click.clone()
                class="p-2 relative flex flex-col gap-1 bg-cover place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
                style:background-image=style_bg_img.clone()
                >
                  <TrashCanBtn author_username post_files_state file=file.clone()/>
                </div>

            }
        }
    };

    let view_cloned = move || {
        view! {
            <a
                on:click=on_click.clone()
                href=link.clone()
                class="p-2 relative flex flex-col gap-1 bg-cover place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05  "
                style:background-image=style_bg_img.clone()
            >
                <div
                    class="text-center z-[2] bg-base03 px-[0.5rem] text-base08 rounded-full absolute left-[0] top-[0] transform -translate-x-1/2 -translate-y-1/2 ">
                    {file_index}
                </div>
                <TrashCanBtn
                  author_username=author_username.clone()
                  post_files_state=post_files_state.clone()
                  file=file.clone()
                />
            </a>
        }.into_any()
    };

    // let v = Show(ShowProps {
    //     // children: { TypedChildrenFn(Arc::new(move || div())) },
    //     children: TypedChildrenFn::from(div()),
    //     when: move || false,
    //     fallback: { div() },
    // });
    //
    // <Floater fix_leptos_please=view_cloned.clone().into() />
    view! {
        <Show when=when_is_link fallback >
            <Floater zones floater_index=file_index fix_leptos_please=view_cloned.clone().into() />
        </Show>

    }
}

#[component]
pub fn FileProcessingPreview(
    #[prop(into)] author_username: Signal<String>,
    post_files_state: PostImagesState,
    file: ArcRwSignal<ParsedPostImage>,
) -> impl IntoView {
    let name = {
        let file = file.clone();
        move || file.with(|v| v.name.clone())
    };
    let spawner = Spawner::new();
    let proccess_state = ProccessingState::new(file.clone());

    let _ = interval::new(
        move |handle| {
            let proccess_state = proccess_state.clone();
            spawner.spawn(async move {
                proccess_state.check(0).await;
            });
        },
        Duration::from_secs(1),
    )
    .inspect_err(|err| error!("{err}"));

    view! { <div
            class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { name }
              </p>
              <p class="text-[0.7rem]">
                  "proccessing..."
              </p>
              <TrashCanBtn author_username post_files_state file/>
            </div>
    }
}

#[component]
pub fn FileErrorPreview(
    #[prop(into)] author_username: Signal<String>,
    post_files_state: PostImagesState,
    file: ArcRwSignal<ParsedPostImage>,
) -> impl IntoView {
    let name = {
        let file = file.clone();
        move || file.with(|v| v.name.clone())
    };
    let err = {
        let file = file.clone();
        move || file.with(|v| v.err.clone())
    };

    view! { <div
            id="previw_add"
            class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { name }
              </p>
              <p class="text-[0.7rem]">
                  { err }
              </p>
              <TrashCanBtn author_username post_files_state file/>
            </div>
    }
}

#[component]
pub fn TrashCanBtn(
    #[prop(into)] author_username: Signal<String>,
    post_files_state: PostImagesState,
    file: ArcRwSignal<ParsedPostImage>,
) -> impl IntoView {
    let page = PageState::get();
    let spawner = Spawner::new();
    let on_click = move |_e: MouseEvent| {
        let file = file.clone();
        spawner.spawn(async move {
            let client = create_client();
            post_files_state.remove_image(&client, file).await;
        });
    };
    let when_is_owner = move || page.acc_username() == author_username.get();

    view! {
        <Show when=when_is_owner>
            <button on:click=on_click.clone()>
                <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
            </button>
        </Show>
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
            for=move||fn_for()
            class=move ||  {
                // let hash = location.hash.get();
                // trace!("hash: {hash}");
                format!("text-[2rem] grid place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05")
            }
            // style:background-image=move || format!("url(\"{url}\")")
            >"+"</label>
    }
}
