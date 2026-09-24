use super::component_edit_area::EditArea;
use super::upload_state::UploadState;
use crate::{
    Errs, Floater, LengthCounter, PageState, SVGSpinner, SVGTrash, Zone, ZoneData, ZoneStage,
    hook::{
        EventListener, ParsedPostImage, ParsedPostImageState, PostImageId, PostImagesState, Spawner,
    },
    page::{create_client, upload::component_edit_text::ValidState},
};
use catsquad_log::prelude::*;
use catsquad_shared::{arr_remove_and_insert, link_relative_post_thumbnail_bytes_get_by_hash};
use catsquad_web_utils::prelude::*;
use leptos::{ev, html::div, prelude::*, task::spawn_local};
use std::{sync::Arc, time::Duration};
use wasm_bindgen::JsCast;
use web_sys::{File, HtmlElement, HtmlInputElement, MouseEvent};

// TODO
// add cancel upload
// add proccesed interval check
// fix styling

#[component]
pub fn ImagesEdit(
    #[prop(into)] author_username: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] post_id: Signal<i64>,
    #[prop(optional, into)] images: RwSignal<Vec<ArcRwSignal<ParsedPostImage>>>,
) -> impl IntoView {
    let page = PageState::get();
    // let on_link = move |f| format!("");

    let spawner = Spawner::new();
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

    let is_loading_fn = move || {
        spawner.is_busy.get()
            || images.with(|v| {
                v.iter()
                    .any(|v| v.with_untracked(|v| v.state != ParsedPostImageState::Completed))
            })
    };

    // let on_click = move |e: MouseEvent| {
    //     //
    // };

    // let on_link = move |v: ArcRwSignal<ParsedPostImage>| String::new();
    // let disabled = false;
    // <ImagesView  disabled=image_view_disabled on_link />
    // <Show when=is_loading_fn>
    //     <SVGSpinner class=move||"size-4"/>
    // </Show>
    // <div class="flex justify-between ">
    //     <LengthCounter
    //         id=move||format!("{}_counter", title_clone2)
    //         counter_current=move||input_text.with(|v|v.trim().len())
    //         counter_max=move||max_length
    //         />
    //     <div class=move||format!("{}", saved_text_color())>{saved_text}</div>
    // </div>
    let saved_text = move || {
        if is_loading_fn() {
            if is_valid() == ValidState::Error {
                "error."
            } else {
                "saving..."
            }
        } else {
            "saved."
        }
    };
    let saved_text_color = move || {
        if is_loading_fn() {
            if is_valid() == ValidState::Error {
                "text-base08"
            } else {
                "text-base0A"
            }
        } else {
            "text-base0B"
        }
    };

    let saved_class = move || format!("{}", saved_text_color());
    let total_size_text = move || {
        images.with(|v| {
            v.iter()
                .map(|v| v.with_untracked(|v| v.size))
                .reduce(|acc, e| acc + e)
                .map(|total_size| bytes_to_str(total_size))
                .unwrap_or_else(|| "0".to_string())
        })
    };
    let max_user_storage = move || bytes_to_str(page.acc_max_storage_bytes() as u64);

    // let required_text_color = move || match is_valid() {
    //     ValidState::Valid => "text-base0B",
    //     ValidState::Error => "text-base08",
    //     ValidState::Empty => match required {
    //         true => "text-base08",
    //         false => "text-base0A",
    //     },
    // };

    view! {
        <div class="flex flex-col gap-2">
            <p class="text-[1.3rem] text-base0F  ">
            "Images"
            </p>
            <EditArea
                    class="flex flex-col gap-2"
                    required=false
                    is_valid=move||is_valid()
                >
                    <div
                        class="flex flex-wrap gap-4 "
                    >
                        <ImagesView spawner disabled=is_loading_fn author_username post_id images/>
                    </div>
                    <div class="flex justify-between ">
                        <p>{total_size_text}" / "{max_user_storage}</p>
                        <div class=saved_class >{saved_text}</div>
                    </div>
            </EditArea>
        </div>
    }
}

#[component]
pub fn ImagesView(
    spawner: Spawner,
    #[prop(into)] author_username: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] post_id: Signal<i64>,
    #[prop(optional, into)] images: RwSignal<Vec<ArcRwSignal<ParsedPostImage>>>,
) -> impl IntoView {
    // let spawner = Spawner::new();

    let page = PageState::get();
    let input_files = NodeRef::new();
    let images_state = PostImagesState::new(post_id, images);
    let zones = ZoneData::new();
    let images = images_state.images;
    let is_disabled = move || disabled.get() || spawner.is_busy.get();

    let on_file_change = move |e| {
        trace!("on_file_change");
        let Some(new_files) = (input_files.get_untracked() as Option<HtmlInputElement>)
            .and_then(|f: HtmlInputElement| f.files())
            .map(|f| f.get_files())
        else {
            warn!("upload canceled, failed to get files");
            return;
        };

        let parset_files = images_state.set_images(new_files.clone());
        for (parset_file, new_file) in parset_files.into_iter().zip(new_files) {
            spawn_local(async move {
                let client = create_client();
                images_state
                    .update_image(&client, new_file, parset_file)
                    .await;
            });
        }
    };

    let on_drop = move |(zone_index, float_index)| {
        trace!("on_drop {} {}", zone_index, float_index);
        let post_id = post_id.get_untracked();
        spawner.spawn(async move {
            let client = create_client();
            let result = client
                .post_update_image_ord(post_id, float_index, zone_index)
                .send()
                .await
                .into_json()
                .await;
            match result {
                Ok(_) => {
                    images.update(|v| {
                        arr_remove_and_insert(v, float_index, zone_index);
                    });
                }
                Err(err) => {
                    error!("{err}");
                    //
                }
            }
            // client.orde
        });
    };

    let view_files = move || {
        trace!("view_files updating");
        let images_len = images.with(|v| v.len());
        images
            .get()
            .into_iter()
            .enumerate()
            .map(|(index, image)| {
                let block = match image.with(|v| v.state.clone()) {
                    ParsedPostImageState::Queue => view! {
                        <FileInfoPreview
                            author_username
                            image_index=index
                            images_state
                            message="waiting...".to_string()
                            enable_trashcan=false
                        />
                    }
                    .into_any(),
                    ParsedPostImageState::Removing => view! {
                        <FileInfoPreview
                            author_username
                            image_index=index
                            images_state
                            message="removing...".to_string()
                            enable_trashcan=false
                        />
                    }
                    .into_any(),
                    ParsedPostImageState::Uploading => view! {
                        <FileUploadingPreview
                            image_index=index
                            images_state
                        />
                    }
                    .into_any(),

                    ParsedPostImageState::Processing => view! {
                        <FileProcessingPreview
                            author_username
                            image_index=index
                            images_state
                        />
                    }
                    .into_any(),
                    ParsedPostImageState::Completed => view! {
                        <FileCompletedPreview
                            author_username
                            image_index=index
                            images_state
                            disabled=is_disabled
                            zones
                        />
                    }
                    .into_any(),
                    ParsedPostImageState::Error => view! {
                        <FileInfoPreview
                            author_username
                            image_index=index
                            images_state
                            message=String::new()
                            enable_trashcan=true
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

    let when_enabled = move || !disabled.get();
    let when_is_owner = move || page.acc_username() == author_username.get() && when_enabled();
    // let last_index = move || images.with(|v| v.len()).saturating_sub(1);
    // <Zone zone_index=last_index zones on_drop/>
    // let last_zone_id = "zone_1000".to_string();
    // let zone_view = create_zone_view(last_zone_id.clone().into());
    // let last_index = move || images.with(|v| v.len());

    view! {
        { view_files }
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
    // pub file: ArcRwSignal<ParsedPostImage>,
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
    pub fn new() -> Self {
        Self {
            // file,
            check_state: CheckState::default().into(),
        }
    }
    pub async fn check(self, time: u64, image: ArcRwSignal<ParsedPostImage>) {
        // self.check_state.with_untracked(|v| v.stage == ProccessingLoadingStage::Busy);
        // let post_id = self.post_id.get_untracked();

        let file_hash = image.with_untracked(|v| v.hash);
        trace!("checking if file {file_hash} is proccesed");

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
                    image.update(|v| v.state = ParsedPostImageState::Completed);
                }
            }
            Err(err) => {
                error!("{err}");
            }
        }
    }
}

#[component]
pub fn FileUploadingPreview(image_index: usize, images_state: PostImagesState) -> impl IntoView {
    let view_speed_text = move || {
        images_state
            .get(image_index)
            .map(|v| v.with(|v| v.upload_speed_bytes_a_second))
            .map(|v| format!("{}/s", bytes_to_str(v)))
            .unwrap_or_default()
    };
    let view_name_text = move || {
        images_state
            .get(image_index)
            .map(|v| v.with(|v| v.name.clone()))
            .unwrap_or_default()
    };
    let view_percent_text = move || {
        images_state
            .get(image_index)
            .map(|v| v.with(|v| v.uploaded_percentage))
            .map(|v| format!("{}%", v))
            .unwrap_or_default()
    };
    let view_size_text = move || {
        images_state
            .get(image_index)
            .map(|v| v.with(|v| (v.uploaded_bytes, v.size)))
            .map(|(uploaded_bytes, size)| {
                format!("{}/{}", bytes_to_str(uploaded_bytes), bytes_to_str(size))
            })
            .unwrap_or_default()
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
                          style:width=view_percent_text
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
    author_username: Signal<String>,
    image_index: usize,
    images_state: PostImagesState,
    #[prop(into)] disabled: Signal<bool>,
    zones: ZoneData,
) -> impl IntoView {
    let when_enabled = move || !disabled.get();

    let link = {
        move || {
            images_state
                .get(image_index)
                .map(|v| PostImageId::new(v.with(|v| v.hash)).to_hashtag())
                .unwrap_or_default()
        }
    };
    let image_size = {
        move || {
            images_state
                .get(image_index)
                .map(|v| v.with(|v| bytes_to_str(v.size)))
                .unwrap_or_default()
        }
    };
    let style_bg_img = move || {
        images_state
            .get(image_index)
            .map(|v| {
                format!(
                    "url(\"{}\")",
                    link_relative_post_thumbnail_bytes_get_by_hash(
                        images_state.post_id.get(),
                        v.with(|v| v.hash)
                    )
                )
            })
            .unwrap_or_default()
    };
    let image_index_view = move || {
        view! {
            <div
                class="text-center z-[2] bg-base03 px-[0.5rem] text-base05 rounded-full absolute left-[0] top-[0] transform -translate-x-1/2 -translate-y-1/2 ">
                {image_index}
            </div>
        }
    };
    let image_size_view = move || {
        view! {
            <div
                class="text-center bg-base03 px-[0.5rem] text-base05 rounded-full absolute left-[0] top-[100%] transform -translate-y-full ">
                {image_size}
            </div>
        }
    };
    let inner_view = move || {
        view! {
            {image_index_view}
            <Show when=when_enabled>
                {image_size_view}
                <TrashCanBtn author_username images_state image_index />
            </Show>
        }
    };
    let when_use_div = move || !disabled.get();
    let when_use_link = move || disabled.get();

    view! {
        <Floater zones floater_index=image_index disabled=disabled >
            <Show when=when_use_div >
                <div
                    class="p-2 relative flex flex-col gap-1 bg-cover place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
                    style:background-image=style_bg_img
                >
                    {inner_view.clone()}
                </div>
            </Show>
            <Show when=when_use_link >
                <a
                    href=link
                    class="p-2 relative flex flex-col gap-1 bg-cover place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05  "
                    style:background-image=style_bg_img
                >
                    {inner_view.clone()}
                </a>
            </Show>

        </Floater>
    }
}

#[component]
pub fn FileProcessingPreview(
    author_username: Signal<String>,
    image_index: usize,
    images_state: PostImagesState,
) -> impl IntoView {
    let spawner = Spawner::new();
    let proccess_state = ProccessingState::new();

    // TODO handle index missmatch

    let _ = interval::new(
        move |handle| {
            let Some(image) = images_state.get(image_index) else {
                warn!("image {image_index} not found");
                let _ = handle.clear().inspect_err(|err| error!("{err}"));
                return;
            };
            let proccess_state = proccess_state.clone();
            spawner.spawn(async move {
                proccess_state.check(0, image).await;
            });
        },
        Duration::from_secs(1),
    )
    .inspect_err(|err| error!("{err}"));

    view! {
        <FileInfoPreview
            message="proccessing...".to_string()
            enable_trashcan=false
            author_username
            images_state
            image_index
        />
    }
}

#[component]
pub fn FileInfoPreview(
    author_username: Signal<String>,
    #[prop(into)] image_index: Signal<usize>,
    images_state: PostImagesState,
    message: String,
    enable_trashcan: bool,
) -> impl IntoView {
    let name = move || {
        images_state
            .get(image_index.get())
            .map(|v| v.with(|v| v.name.clone()))
            .unwrap_or_default()
    };
    let err = move || {
        images_state
            .get(image_index.get())
            .map(|v| v.with(|v| v.err.clone()))
            .unwrap_or_default()
    };
    let when_message = message.is_empty();
    let when_message = move || when_message;
    let when_trashable = move || enable_trashcan;

    view! { <div
            class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { name }
              </p>
              <Errs error=err />
              <Show when=when_message>
                  <p class="text-[0.7rem]">
                      { message.clone() }
                  </p>
              </Show>
              <Show when=when_trashable >
                  <TrashCanBtn author_username images_state image_index />
              </Show>
            </div>
    }
}

#[component]
pub fn TrashCanBtn(
    author_username: Signal<String>,
    #[prop(into)] image_index: Signal<usize>,
    images_state: PostImagesState,
) -> impl IntoView {
    // let file = RwSignal::new(file);
    let page = PageState::get();
    let spawner = Spawner::new();
    let on_click = move |_e: MouseEvent| {
        let Some(img) = images_state.get(image_index.get()) else {
            return;
        };
        spawner.spawn(async move {
            let client = create_client();
            images_state.remove_image(&client, img).await;
        });
    };
    let when_is_owner = move || page.acc_username() == author_username.get();
    // <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full z-[3] absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />

    view! {
        <Show when=when_is_owner>
            <button on:click=on_click.clone()>
                <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full z-[3] absolute left-[100%] top-[100%] transform -translate-x-full -translate-y-full size-[2.0rem]" />
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

    view! { <label
            for=move||fn_for()
            class=move ||  {
                format!("text-[2rem] grid place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05")
            }
            >"+"</label>
    }
}

// style:background-image=move || format!("url(\"{url}\")")
// let hash = location.hash.get();
// trace!("hash: {hash}");
// #[component]
// pub fn FileQueuePreview(file: ArcRwSignal<ParsedPostImage>) -> impl IntoView {
//     let name = {
//         let file = file.clone();
//         move || file.with(|v| v.name.clone())
//     };

//     // let size = bytes_to_str(file.size as u64);

//     view! { <div
//             class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
//             >
//               <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
//                   { name }
//               </p>
//               <p class="text-[0.7rem]">
//                   "preparing..."
//               </p>
//             </div>
//     }
// }
// let location = use_location();
// let name = {
//     let file = file.clone();
//     move || file.with(|v| v.name.clone())
// };

// let link = {
//     let file = file.clone();
//     move || {
//         if let Some(f) = on_link {
//             f.run(file.clone())
//         } else {
//             String::new()
//         }
//     }
// };

// let on_click = move |e: MouseEvent| {
//     if let Some(f) = on_click {
//         f.run(e)
//     }
// };

// let when_is_link = {
//     let link = link.clone();
//     move || !link().is_empty() && disabled.get()
// };

// let view_cloned = move || {
//     let fallback = fallback.clone();
//     // let when_is_link = when_is_link.clone();
//     let inner_view = inner_view.clone();
//     let link = PostImageId::new(v.hash).to_hashtag();
//     let style_bg_img = style_bg_img.clone();
//     view! {
//         <Show when=when_is_link fallback >
//             <a
//                 on:click=on_click.clone()
//                 href=link.clone()
//                 class="p-2 relative flex flex-col gap-1 bg-cover place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05  "
//                 style:background-image=style_bg_img.clone()
//             >
//                 {inner_view.clone()}
//             </a>
//         </Show>
//     }.into_any()
// };

// let v = Show(ShowProps {
//     // children: { TypedChildrenFn(Arc::new(move || div())) },
//     children: TypedChildrenFn::from(div()),
//     when: move || false,
//     fallback: { div() },
// });
//
// <Floater fix_leptos_please=view_cloned.clone().into() />
