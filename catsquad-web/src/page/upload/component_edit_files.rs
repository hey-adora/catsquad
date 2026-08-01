use super::component_edit_area::EditArea;
use super::upload_state::UploadState;
use crate::{
    Errs, SVGTrash,
    hook::Spawner,
    page::{create_client, upload::component_edit_text::ValidState},
};
use catsquad_web_utils::prelude::*;
use leptos::prelude::*;
use web_sys::{File, HtmlInputElement, MouseEvent};

#[derive(Clone, Copy)]
pub enum UploadProgressState {
    Queue,
    Uploading,
    Completed,
}

#[derive(Clone)]
pub struct ParsedFile {
    pub file: File,
    pub name: String,
    pub size: f64,
    pub state: UploadProgressState,
}

#[component]
pub fn ImagesEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
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
            v.extend(new_files.clone().into_iter().map(|file| {
                let name = file.name();
                let size = file.size();

                ParsedFile {
                    file,
                    name,
                    size,
                    state: UploadProgressState::Queue,
                }
            }))
        });

        let client = create_client();
        let post_key = "6disschr96ma2jivnnyr";
        spawner.spawn(async move {
            let result = client
                .post_update_file_add(post_key, new_files)
                .await
                .send()
                .await
                .into_res()
                .await;

            //
        });

        //
    };
    let view_files = move || {
        files
            .get()
            .into_iter()
            .map(|file| match file.state {
                UploadProgressState::Queue => view! {
                    <FileQueuePreview
                        file
                    />
                }
                .into_any(),
                UploadProgressState::Uploading => view! {
                    <FileUploadPreview
                        file
                    />
                }
                .into_any(),
                UploadProgressState::Completed => view! {
                    <FileCompletedPreview
                        file
                    />
                }
                .into_any(),
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
                class=move||"flex gap-4"
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
pub fn FileQueuePreview(file: ParsedFile) -> impl IntoView {
    let name = file.name;
    let size = bytes_to_str(file.size as u64);

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
              <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
            </div>
    }
}

#[component]
pub fn FileUploadPreview(file: ParsedFile) -> impl IntoView {
    let name = file.name;
    let size = bytes_to_str(file.size as u64);

    view! { <div
            id="previw_add"
            class="p-2 relative flex flex-col gap-1 place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
            >
              <p class="text-[0.8rem] max-w-[100%] max-h-[100%] break-all overflow-hidden text-ellipsis">
                  { name }
              </p>
              <p class="text-[0.7rem]">
                  "2.78 MB/s"
              </p>
              <div class="bg-base01 w-full rounded-full ">
                  <p
                      class="bg-base05 text-center font-bold rounded-full text-base01 text-[0.9rem]"
                      style:width="45%"
                      >
                   "45%"
                  </p>
              </div>
              <p class="text-[0.7rem]">
                  "0/"{size}
              </p>
              <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
            </div>
    }
}

#[component]
pub fn FileCompletedPreview(file: ParsedFile) -> impl IntoView {
    let name = file.name;
    let size = bytes_to_str(file.size as u64);

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
              <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
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
                format!("text-[2rem] grid place-items-center size-[8rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05")
            }
            // style:background-image=move || format!("url(\"{url}\")")
            >"+"</label>
    }
}
