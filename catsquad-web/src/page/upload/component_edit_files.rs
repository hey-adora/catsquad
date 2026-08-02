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

// #[derive(Clone, Copy)]
// pub enum UploadProgressState {
//     Queue,
//     Uploading,
//     Completed,
// }

// #[derive(Clone)]
// pub struct ParsedFile {
//     pub file: File,
//     pub name: String,
//     pub size: f64,
//     pub state: UploadProgressState,
// }

#[component]
pub fn ImagesEdit(upload: UploadState) -> impl IntoView {
    let spawner = Spawner::new();
    let input_files = NodeRef::new();
    let files = upload.files;
    // let files = RwSignal::<Vec<ParsedFile>>::new(Vec::new());
    // let client = create_client();
    // Effect::new(move || {
    //     spawner.spawn(async move {
    //         let result = client
    //             .post_update_file_add(post_key, new_files)
    //             .await
    //             .send()
    //             .await
    //             .into_res()
    //             .await;
    //     });
    // });
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
        files.update(|v| {
            for new_file in new_files {
                let web_file: ParsedPostFile = new_file.clone().into();
                // let file_name = web_file.name.clone();
                let file = ArcRwSignal::new(web_file);

                v.push(file.clone());

                let client = create_client();
                let post_key = post_key.clone();
                let file_vec = vec![new_file];

                spawn_local(async move {
                    let result = client
                        .post_update_file_add(post_key, file_vec)
                        .await
                        .on_progress({
                            let file = file.clone();
                            move |stats| {
                                file.update(|parsed_file| {
                                    parsed_file.state = ParsedPostFileState::Uploaded;
                                    parsed_file.uploaded_bytes = stats.completed_bytes;
                                    parsed_file.upload_speed_bytes_a_second =
                                        stats.upload_speed_bytes;
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
                                file.update(|file| {
                                    file.state = ParsedPostFileState::Error;
                                    file.err = "received wrong response".to_string();
                                });
                                return;
                            }

                            let Some(_received) = received_post.first() else {
                                return;
                            };

                            file.update(|file| {
                                file.state = ParsedPostFileState::Uploaded;
                            });

                            // post_files.update(|post_files| {
                            //     let Some(pos) = post_files.iter().position(|v| *v == file) else {
                            //         warn!("file {} is gone", file_name);
                            //         return;
                            //     };
                            //     trace!("upoloading file REMOVED {pos}");
                            //     post_files.remove(pos);

                            //     for received_file in received_post.file {
                            //         let Some(pos) = post_files.iter().position(|v| {
                            //             v.with_untracked(|v| v.name == received_file.hash)
                            //         }) else {
                            //             trace!("upoloading file ADDED {pos}");
                            //             post_files.push(ArcRwSignal::new(received_file.into()));
                            //             return;
                            //         };

                            //         trace!("upoloading file REPLACE {pos}");
                            //         post_files[pos] = ArcRwSignal::new(received_file.into());
                            //     }
                            // });
                        }
                        Err(err) => {
                            file.update(|file| {
                                file.state = ParsedPostFileState::Error;
                                file.err = err.to_string();
                            });
                        }
                    }
                });
            }
            // v.extend(new_files.clone().into_iter().map(|file| {
            //     let name = file.name();
            //     let size = file.size();

            //     ParsedPostFile {
            //         name,
            //         size,
            //         state: ParsedPostFileState::Queue,
            //         err: String::new(),
            //     }
            // }))
        });

        // let client = create_client();
        // let post_key = "6disschr96ma2jivnnyr";
        // spawner.spawn(async move {
        //     let result = client
        //         .post_update_file_add(post_key, new_files)
        //         .await
        //         .send()
        //         .await
        //         .into_res()
        //         .await;

        //     //
        // });

        //
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
                ParsedPostFileState::Uploading => view! {
                    <FileUploadingPreview
                        file
                    />
                }
                .into_any(),

                ParsedPostFileState::Uploaded => view! {
                    <FileUploadedPreview
                        file
                    />
                }
                .into_any(),
                ParsedPostFileState::Proccesed => view! {
                    <FileProccesedPreview
                        file
                    />
                }
                .into_any(),
                ParsedPostFileState::Error => view! {
                    <FileErrorPreview
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
              <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
            </div>
    }
}

#[component]
pub fn FileUploadingPreview(file: ArcRwSignal<ParsedPostFile>) -> impl IntoView {
    let name = {
        let file = file.clone();
        move || file.with(|v| v.name.clone())
    };
    let size = {
        let file = file.clone();
        move || file.with(|v| bytes_to_str(v.size as u64))
    };
    let upload_percentage = {
        let file = file.clone();
        move || file.with(|v| bytes_to_str(v.uploaded_percentage as u64))
    };

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
                  {upload_percentage}"/"{size}
              </p>
              <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
            </div>
    }
}

#[component]
pub fn FileUploadedPreview(file: ArcRwSignal<ParsedPostFile>) -> impl IntoView {
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
              <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
            </div>
    }
}

#[component]
pub fn FileProccesedPreview(file: ArcRwSignal<ParsedPostFile>) -> impl IntoView {
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
              <SVGTrash class="bg-base03 p-[0.35rem] text-base08 rounded-full absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2 size-[2.0rem]" />
            </div>
    }
}

#[component]
pub fn FileErrorPreview(file: ArcRwSignal<ParsedPostFile>) -> impl IntoView {
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
