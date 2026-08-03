use super::component_edit_area::EditArea;
use super::upload_state::UploadState;
use crate::{
    Errs, SVGTrash,
    hook::Spawner,
    page::{
        create_client,
        upload::{
            component_edit_text::ValidState,
            upload_state::{ParsedPostFile, ParsedPostFileState},
        },
    },
};
use catsquad_log::prelude::*;
use catsquad_web_utils::prelude::*;
use leptos::{prelude::*, task::spawn_local};
use web_sys::{File, HtmlInputElement, MouseEvent};

// TODO
// add cancel upload
// add proccesed interval check
// fix styling

#[component]
pub fn ImagesEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
    let input_files = NodeRef::new();
    let files = upload.files;

    let on_file_change = move |e| {
        let Some(new_files) = (input_files.get_untracked() as Option<HtmlInputElement>)
            .and_then(|f: HtmlInputElement| f.files())
            .map(|f| f.get_files())
        else {
            warn!("upload canceled, failed to get files");
            return;
        };
        // let post_files = upload.files;
        let post_key = upload.post_key.get_value();
        if post_key.is_empty() {
            warn!("upload canceled, post_key is empty");
            return;
        }
        let parset_files = upload.set_files(new_files.clone());
        for (parset_file, new_file) in parset_files.into_iter().zip(new_files) {
            spawn_local(async move {
                let client = create_client();
                upload.update_file(&client, new_file, parset_file).await;
            });
        }
    };

    let view_files = move || {
        // upload.
        files
            .get()
            .into_iter()
            .map(|file| match file.with(|v| v.state.clone()) {
                ParsedPostFileState::Queue => view! {
                    <FileQueuePreview
                        file
                    />
                }
                .into_any(),
                ParsedPostFileState::Removing => view! {
                    <FileQueuePreview
                        file
                    />
                }
                .into_any(),
                ParsedPostFileState::Uploading => view! {
                    <FileUploadingPreview
                        file
                    />
                }
                .into_any(),

                ParsedPostFileState::Uploaded => view! {
                    <FileUploadedPreview
                        upload
                        file
                    />
                }
                .into_any(),
                ParsedPostFileState::Proccesed => view! {
                    <FileProccesedPreview
                        upload
                        file
                    />
                }
                .into_any(),
                ParsedPostFileState::Error => view! {
                    <FileErrorPreview
                        upload
                        file
                    />
                }
                .into_any(),
            })
            .collect_view()
            .into_any()
    };
    let is_valid = move || {
        let state = files.with(|v| {
            if v.is_empty() {
                return ValidState::Empty;
            }
            let has_err = v
                .iter()
                .any(|v| v.with(|v| v.state == ParsedPostFileState::Error));
            if has_err {
                return ValidState::Error;
            }
            ValidState::Valid
        });
        state
    };
    view! {
        <div class="flex flex-col gap-2">
            <p class="text-[1.3rem] text-base0F ">"Images"</p>
            <Errs error=move||upload.err_general.get() />
            <EditArea
                class=move||"flex flex-wrap gap-4 "
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
pub fn FileQueuePreview(file: ArcRwSignal<ParsedPostFile>) -> impl IntoView {
    let name = {
        let file = file.clone();
        move || file.with(|v| v.name.clone())
    };
    // let size = bytes_to_str(file.size as u64);

    view! { <div
            id="previw_add"
            class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { name }
              </p>
              <p class="text-[0.7rem]">
                  "waiting..."
              </p>
            </div>
    }
}

#[component]
pub fn FileUploadingPreview(file: ArcRwSignal<ParsedPostFile>) -> impl IntoView {
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
            id="previw_add"
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
pub fn FileUploadedPreview(
    upload: UploadState,
    file: ArcRwSignal<ParsedPostFile>,
) -> impl IntoView {
    let name = {
        let file = file.clone();
        move || file.with(|v| v.name.clone())
    };

    view! { <div
            id="previw_add"
            class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { name }
              </p>
              <p class="text-[0.7rem]">
                  "completed"
              </p>
              <TrashCanBtn upload file/>
            </div>
    }
}

#[component]
pub fn FileProccesedPreview(
    upload: UploadState,
    file: ArcRwSignal<ParsedPostFile>,
) -> impl IntoView {
    let name = {
        let file = file.clone();
        move || file.with(|v| v.name.clone())
    };

    view! { <div
            id="previw_add"
            class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { name }
              </p>
              <p class="text-[0.7rem]">
                  "proccessed"
              </p>
              <TrashCanBtn upload file/>
            </div>
    }
}

#[component]
pub fn FileErrorPreview(upload: UploadState, file: ArcRwSignal<ParsedPostFile>) -> impl IntoView {
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
              <TrashCanBtn upload file/>
            </div>
    }
}

#[component]
pub fn TrashCanBtn(upload: UploadState, file: ArcRwSignal<ParsedPostFile>) -> impl IntoView {
    let spawner = Spawner::new();
    let on_click = move |_e: MouseEvent| {
        let file = file.clone();
        spawner.spawn(async move {
            let client = create_client();
            upload.remove_file(&client, file).await;
        });
    };

    view! {
        <button on:click=on_click>
            <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
        </button>
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
                format!("text-[2rem] grid place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05")
            }
            // style:background-image=move || format!("url(\"{url}\")")
            >"+"</label>
    }
}
