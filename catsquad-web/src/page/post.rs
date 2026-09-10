use catsquad_log::prelude::*;
use catsquad_shared::{MAX_POST_DESCRIPTION_LENGTH, MAX_POST_TAGS_LENGTH, MAX_POST_TITLE_LENGTH};
use catsquad_web_utils::prelude::*;
use leptos::Params;
use leptos::{html, prelude::*};
use leptos_router::hooks::{use_location, use_params};
use leptos_router::params::Params;
use web_sys::MouseEvent;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};

use crate::PageState;
use crate::hook::Spawner;
use crate::page::create_client;
use crate::{AutoTextArea, BtnPrimary, BtnSecondary, Errs, Nav, SVGTrash};
use component_favorite::Favorite;
use post_api::PostApi;

mod comments_api;
mod comments_basic;
mod component_comment;
mod component_comments;
mod component_edit_btn;
mod component_favorite;
mod component_length_counter;
mod component_post_title;
mod post_api;
mod post_like_state;

use component_comments::Comments;
use component_edit_btn::EditSaveCancel;
use component_length_counter::LengthCounter;
use component_post_title::PostTitle;

#[derive(Params, PartialEq, Clone)]
pub struct PostParams {
    pub username: Option<String>,
    pub post: Option<String>,
}

#[component]
pub fn Post() -> impl IntoView {
    // TODO add max limit for description and tags and other stuff
    let main_ref = NodeRef::new();
    // let api_post = ApiWeb::new();
    // let api_comments = ApiWeb::new();
    let global_state = PageState::get();

    let param = use_params::<PostParams>();
    // let param_username = move || param.read().as_ref().ok().and_then(|v| v.username.clone());
    let param_post_id = Memo::new(move |_| {
        param
            .read()
            .as_ref()
            .ok()
            .and_then(|v| v.post.clone())
            .and_then(|v| i64::from_str_radix(&v, 10).ok())
    });

    let location = use_location();

    let spawner_post = Spawner::new();
    let post_api = PostApi::new();
    let edit_tags_input = NodeRef::<html::Textarea>::new();
    let description_input_editor = NodeRef::<html::Textarea>::new();

    let post_user_username = move || post_api.author_username.get();
    let post_id = move || param_post_id.get().unwrap_or_default();

    // use_text_counter(description_input_editor, post_api.live_description_length);
    // use_text_counter(edit_tags_input, post_api.live_tags_length);

    Effect::new(move || {
        let Some(post_id) = param_post_id.get() else {
            return;
        };

        spawner_post.spawn(async move {
            let client = create_client();
            post_api.get(&client, post_id).await;
        });
    });

    // Effect::new(move || {
    //     // let (Some(description_ref), Some(edit_description_ref)) = (
    //     //     ) else {
    //     //     return;
    //     // };
    //     description_mutation.disconnect();

    //     // if let Some(elm) = description_input.get() {
    //     //     description_mutation.observe(
    //     //         elm,
    //     //         MutationObserverOptions::new()
    //     //             .character_data()
    //     //             .set_child_list()
    //     //             .subtree(),
    //     //     );
    //     // }

    //     if let Some(elm) = description_input_editor.get() {
    //         description_mutation.observe(
    //             elm,
    //             MutationObserverOptions::new()
    //                 .character_data()
    //                 .set_child_list()
    //                 .subtree(),
    //         );
    //     }

    //     // trace!("comments basic observe");
    //     // spawner_comments.spawn(comment_basic.observe_only(comment_container_ref.into(), post_id));
    // });

    // let post_like = use_post_like(param_post);
    // let post_like_fn = move || {
    //     post_like.on_like.with_value(|f| {
    //         (f)();
    //     });
    // };
    // let is_post_liked_fn = move || post_like.stage.with_value(|f| (f)() == LikeState::Liked);
    // let post_like_btn_style = move || {
    //     format!(
    //         "border-2 text-[1.3rem] font-bold px-4 py-1 {}",
    //         match post_like.stage.run() {
    //             LikeState::Liked => "border-base01 bg-base05 text-base01",
    //             LikeState::Unliked =>
    //                 "border-base05 bg-base01 text-base05 hover:bg-base05 hover:text-base01",
    //             LikeState::Loading => "border-base05 bg-base01 text-base05",
    //         }
    //     )
    // };
    // let post_like_btn_text = move || match post_like.stage.run() {
    //     LikeState::Liked => "Un-Favorite",
    //     LikeState::Unliked => "Favorite",
    //     LikeState::Loading => "Loading",
    // };

    let selected_img = move || -> AnyView {
        let hash = location.hash.get();
        let imgs_links = post_api.imgs_links.get();
        let selected_n = if hash.len() > 3 {
            usize::from_str_radix(&hash[3..], 10).unwrap_or_default()
        } else {
            0
        };
        let Some((selected_url, selected_ratio)) = imgs_links.get(selected_n).cloned() else {
            return view! {
                <p>
                    "No Image"
                </p>
                // <img id=move || format!("id0") class="max-h-full" src=selected_url />
            }
            .into_any();
        };

        view! {
            <img id=move || format!("id{selected_n}") class="max-h-full" src=selected_url />
        }
        .into_any()
    };

    let imgs = move || -> Vec<AnyView> {
        let imgs = post_api.imgs_links.get();
        if imgs.is_empty() {
            let view = view! {
                    <div style:aspect-ratio="1.0" class=" w-full h-[50%] grid place-items-center bg-base02">
                        <p id="id0" >"No Images"</p>
                        // <img id=move || format!("id{i}") class="" src=url />
                    </div>

            }
            .into_any();
            return vec![view];
        }
        imgs
                .into_iter()
                .enumerate()
                .map(|(i, (url, ratio))| view! {
                    <div style:aspect-ratio=ratio.to_string() class="w-full grid place-items-center bg-base02">
                        <img id=move || format!("id{i}") class="" src=url />
                    </div>
                }.into_any())
                .collect_view()
    };

    let edit_description_mode_toggle = move || {
        let description_len = post_api.description.with_untracked(|v| v.len());
        post_api.live_description_length.set(description_len);
        post_api.update_description_mode.update(|v| *v = !*v);
    };
    let edit_description_save = move || {
        let (Some(post_key), Some(new_description)) = (
            param_post_id.get(),
            description_input_editor
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        spawner_post.spawn(async move {
            let client = create_client();
            post_api
                .update_description(&client, post_key, new_description)
                .await;
        });
    };
    // let edit_description_cancel = move || {
    //     // post_api.description.update(|_v| ());
    //     edit_description_mode_toggle();
    // };

    // let edit_description_keydown = move |e: Event| {
    //     let v = e
    //         .target()
    //         .map(|v: EventTarget| Into::<HtmlPreElement>::into(JsValue::from(v)) )
    //         .map(|v| v.id())
    //         ;
    //
    //     // let Some(new_description) = (
    //     //     edit_description_input
    //     //         .get_untracked()
    //     //         .and_then(|v: HtmlPreElement| v.text_content()),
    //     // ) else {
    //     //     return;
    //     // };
    //     trace!("wtf description changed {v:?}");
    // };

    // let description = move || {
    //     let mut description = post_api.description.get();
    //     if description.is_empty() {
    //         description.push_str("No description.");
    //     }
    //     description
    // };

    let description = move || {
        post_api.description.get()
        // let mut a = view! {};
        // post_api
        //     .description
        //     .get()
        //     .split('\n')
        //     .into_iter()
        //     .map(|v| {
        //         view! {
        //             <div class="">
        //                 {v}
        //             </div>
        //         }
        //     })
        //     .collect_view()
    };

    let tags = move || {
        post_api
            .tags
            .get()
            .split_whitespace()
            .into_iter()
            .map(|v| {
                view! {
                    <div class="bg-base02 rounded-full text-[1rem] px-3 py-1">
                        {v}
                    </div>

                }
            })
            .collect_view()
    };

    let edit_tags = move || {
        post_api.update_tags_mode.update(|v| *v = !*v);
    };

    let edit_tags_save = move || {
        let (Some(post_key), Some(tags)) = (
            param_post_id.get(),
            edit_tags_input
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        spawner_post.spawn(async move {
            let client = create_client();
            post_api.update_tags(&client, post_key, tags).await;
        });
    };

    let upload_image = NodeRef::<html::Input>::new();
    let on_upload = move |_| {
        let (Some(files),): (Option<Vec<web_sys::File>>,) = (
            (upload_image.get_untracked())
                .and_then(|f: HtmlInputElement| f.files())
                .map(|f| f.get_files()),
            // upload_title.get_untracked() as Option<HtmlInputElement>,
            // upload_description.get_untracked() as Option<HtmlTextAreaElement>,
            // upload_tags.get_untracked() as Option<HtmlTextAreaElement>,
        ) else {
            return;
        };

        // uploader.upload(&files[..]);

        trace!("files selected: {}", files.len());
    };

    let previews = move || {
        let mut imgs = post_api.imgs_links.get();

        // imgs.push((String::new(), 0.0);

        let mut views = imgs
            .into_iter()
            .enumerate()
            .map(|(i, (url, ratio))| {
                view! {
                    <PreviewImg
                        index=move|| i
                        link=move|| url.clone()
                    />
                }
                .into_any()
            })
            .collect_view();

        let preview_add = {
            // let i = views.len();
            // let id = format!("#id{i}");
            // let id2 = id.clone();

            view! {
                <div>
                    <label
                        id="previw_add"
                        for="image"
                        class="text-[2rem] grid place-items-center h-[5rem] w-[5rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
                        >"+"</label>
                    <input class="absolute z-[-1] opacity-0" on:change=on_upload type="file" id="image" name="image" node_ref=upload_image multiple />
                </div>
            }
        };

        views.push(preview_add.into_any());

        views
    };

    let delete_post = move |_| {
        let Some(post_id) = param_post_id.get() else {
            return;
        };
        spawner_post.spawn(async move {
            let client = create_client();
            post_api.delete(&client, post_id).await;
        });
    };

    // let uploader = FileUpload::new();

    // let show_favorite_btn = move || -> bool {
    //     global_state.is_logged_in().unwrap_or_default() && post
    // };
    // <BtnDelete id=move||"cancel_btn" class=move||"mr-auto" is_loading on_click=on_cancel.clone()>"Cancel"</BtnDelete>

    view! {
        <main node_ref=main_ref class="relative font-hi grid grid-rows-[auto_1fr] h-screen text-base05">
            <Nav/>

            <Show when=move|| post_api.post_state.get().is_not_found() >
                <div class=move || format!("grid place-items-center text-[1.5rem] ")>
                    "Not Found"
                </div>
            </Show>

            <Show when=move|| post_api.post_state.get().is_deleted() >
                <div class=move || format!("grid place-items-center text-[1.5rem] ")>
                    "deleted"
                </div>
            </Show>

            <Show when=move|| {
                let state = post_api.post_state.get();
                state.is_normal() || state.is_loading()
            } >
                <div class=move || format!("flex flex-col lg:grid grid-rows-[auto_1fr] grid-cols-[2fr_1fr] lg:max-h-[calc(100vh-3rem)] gap-2  md:gap-6")>
                    <div class="col-span-2 flex justify-between px-4 md:px-6 ">
                        <div></div>
                        <div>
                            <button on:click=delete_post>
                                <SVGTrash class="size-[1rem] text-base08 "/>
                            </button>
                        </div>
                    </div>
                    <div class="lg:hidden h-[50vh] flex justify-center place-items-center bg-base02" >
                        { selected_img }
                    </div>
                    <div class="hidden lg:flex flex-col gap-2 lg:overflow-y-scroll" >
                        { imgs }
                    </div>
                    <div class="flex flex-col gap-2 md:gap-6 px-4 md:px-6  lg:overflow-y-scroll">
                        <div class="flex justify-start gap-2 flex flex-wrap">
                            { previews }
                        </div>



                        <div class="flex flex-col gap-2">
                            <PostTitle spawner=spawner_post post_api post_id />
                            <div class="flex justify-between place-items-start">
                                <div class="flex gap-2">
                                    <p class="text-[1rem] rounded-full h-[3rem] w-[3rem] bg-base05"></p>
                                    <div class="flex flex-col gap-1">
                                        <div class="flex gap-1">
                                            <p class="text-[1rem] text-base03">"by"</p>
                                            <a href=move || post_api.author_link.get() class="text-[1rem] font-bold text-base0B">{ move || post_api.author_username.get() }</a>
                                        </div>
                                        <p class="text-[1rem]">"9999 followers"</p>
                                    </div>
                                </div>
                                <Favorite post_user_username=post_user_username post_id=post_id />
                                // <Show when=move||global_state.is_logged_in().unwrap_or_default()>
                                //     <BtnSecondary class=move || format!("flex gap-2 place-items-center ") id=move || "btn_favorite" on_click=move|_|post_like_fn()>
                                //         <span class="mt-[0.1rem]">"Favorite"</span>
                                //         <SVGStar class=move||"shrink-0 w-[1.5rem] pb-[0.1rem]" fill=move||is_post_liked_fn() />
                                //     </BtnSecondary>
                                // </Show>
                                // <div>{move || post_api.favorites.get() }" favorites"</div>
                            </div>
                        </div>
                        // <div>
                        //     <AutoTextArea/>
                        // </div>
                        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4">
                            <div class="flex justify-between">
                                <h1 class="text-[1.3rem] text-base0F">"Description"</h1>
                                <div class="flex gap-2 items-center">

                                    <Show when=move || global_state.is_logged_in().unwrap_or_default() >
                                        <Show when=move|| post_api.update_description_mode.get()>
                                            <LengthCounter
                                                counter_current=move||post_api.live_description_length.get()
                                                counter_max=move||MAX_POST_DESCRIPTION_LENGTH
                                            />
                                        </Show>
                                        <EditSaveCancel
                                            id=move || "description"
                                            when=move || post_api.update_description_mode.get()
                                            on_save=move || edit_description_save()
                                            on_cancel=move || edit_description_mode_toggle()
                                            on_edit=move || edit_description_mode_toggle()
                                        />
                                    </Show>
                                </div>
                            </div>

                            <Show when=move || post_api.err_description.with(|v| !v.is_empty()) >
                                <ul id="description_errors" class="ml-[1rem] text-base08 list-disc">
                                    {move || post_api.err_description.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                            </Show>

                            <Show when=move || post_api.update_description_mode.get() fallback=move || view!{
                                <pre
                                    id="post_description"
                                    class=move || format!("whitespace-break-spaces break-all text-ellipsis overflow-hidden padding max-w-[calc(100vw-1rem)] rounded {}",
                                        if post_api.description.with(|v| v.is_empty()) { "text-base03" }  else { "" }

                                        )>
                                    { description }
                                </pre>
                            }>

                            <AutoTextArea
                                id=move||"post_description_editable"
                                node_ref=description_input_editor
                                on_input=move|v:HtmlTextAreaElement| post_api.live_description_length.set(v.value().len())
                                class=move||"bg-base01 text-base05 px-4 py-2 rounded"
                            >
                                { description }
                            </AutoTextArea>
                                // <pre
                                //     id="post_description_editable"
                                //     // on:change=edit_description_keydown
                                //     node_ref=description_input_editor
                                //     contenteditable=true
                                //     class="whitespace-break-spaces break-all text-ellipsis overflow-hidden padding max-w-[calc(100vw-1rem)] bg-base01 text-base05 px-4 py-2 rounded">
                                //     { move || post_api.description.get() }
                                // </pre>
                            </Show>
                        </div>
                        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4">
                            <div class="flex justify-between">
                                <h1 class="text-[1.3rem] text-base0F">"Tags"</h1>
                                <div class="flex gap-2 items-center">

                                    <Show when=move || global_state.is_logged_in().unwrap_or_default() >
                                        <Show when=move|| post_api.update_tags_mode.get()>
                                            <LengthCounter
                                                counter_current=move||post_api.live_tags_length.get()
                                                counter_max=move||MAX_POST_TAGS_LENGTH
                                            />
                                        </Show>
                                        <EditSaveCancel
                                            id=move || "tags"
                                            when=move || post_api.update_tags_mode.get()
                                            on_save=move || edit_tags_save()
                                            on_cancel=move || edit_tags()
                                            on_edit=move || edit_tags()
                                        />
                                    </Show>

                                </div>

                            </div>
                            <Show when=move || post_api.err_tags.with(|v| !v.is_empty()) >
                                <ul class="ml-[1rem] text-base08 list-disc">
                                    {move || post_api.err_tags.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                            </Show>
                            <div class=move || format!("text-ellipsis flex flex-wrap gap-1 overflow-hidden padding max-w-[calc(100vw-1rem)] ",
                                )>

                                <Show when={move || post_api.update_tags_mode.get() } fallback={move || view!{
                                    <Show when={move || post_api.tags.with(|v| !v.is_empty()) } fallback={move || view!{<span class="text-base03">"No tags."</span>} }>
                                        { tags }
                                    </Show>
                                } }>
                                    <AutoTextArea
                                        id=move||"post_tags_editable"
                                        node_ref=edit_tags_input
                                        on_input=move|v:HtmlTextAreaElement| post_api.live_tags_length.set(v.value().len())
                                        class=move||"text-[1.1rem] break-all focus:outline-none! appearance-none border-none resize w-full rounded bg-base01 px-4 py-2"
                                        min_height=100.0
                                    >
                                         {move || post_api.tags.get()}
                                    </AutoTextArea>
                                    // <div contenteditable=true
                                    //      node_ref=edit_tags_input
                                    //      class={move || format!("  ")}>
                                    //      {move || post_api.tags.get()}
                                    // </div>
                                </Show>
                             </div>
                        </div>
                        <Comments post_id />
                    </div>
                </div>
            </Show>


            // TODO probably change 1fr to fixed size or auto or minmax bs
        </main>
    }
}

pub async fn delete_post_img(index: usize) -> Option<()> {
    // self.err_tags.update(|v| v.clear());

    // let api = ApiWebTmp::new()

    // let result =
    //     api
    //     .delete_post_like(post_key, tags)
    //     .send_native()
    //     .await;

    // match result {
    //     Ok(crate::api::ServerRes::Post(v)) => {
    //         self.live_tags_length.set(v.tags.len());
    //         self.tags.set(v.tags);
    //         self.update_tags_mode.set(false);
    //         return Some(());
    //     }
    //     Ok(res) => {
    //         let err = format!("wrong res, expected Post, got {:?}", res);
    //         error!(err);
    //         self.err_tags.set(err);
    //     }
    //     Err(ServerErr::NotFoundErr(Server404Err::NotFound)) => {
    //         self.post_state.set(PostState::NotFound);
    //         self.err_general.set("post not found".to_string());
    //     }
    //     Err(err) => {
    //         let err = format!("unexpected err {:#?}", { err });
    //         error!(err);
    //         self.err_tags.set(err);
    //     }
    // }

    None
}

#[component]
pub fn PreviewImg(
    #[prop(optional, into)] index: Option<Callback<(), usize>>,
    #[prop(optional, into)] link: Option<Callback<(), String>>,
    // #[prop(optional, into)] hash: Option<Callback<(), String>>,
) -> impl IntoView {
    let location = use_location();

    let index_fn = move || index.map(|v| v.run(())).unwrap_or_default();
    let link_fn = move || link.map(|v| v.run(())).unwrap_or_default();
    let id_fn = move || format!("#id{}", index_fn());
    let is_selected = move || {
        let hash = location.hash.get();
        let index = index_fn();
        let id = id_fn();
        id == hash || (hash.is_empty() && index == 0)
    };

    view! {
        <a
            href=id_fn
            class=move || format!("h-[5rem] w-[5rem] bg-base02 bg-cover bg-center rounded-xl {}", if is_selected() {"border-3 border-base05"} else {""})
            style:background-image=move || format!("url(\"{}\")", link_fn())
        >
        </a>
    }
}

#[component]
pub fn PreviewAdd(
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] hash: Option<Callback<(), String>>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
    let location = use_location();

    view! { <button
            id="previw_add"
            // href=id2
            class=move ||  {
                let hash = location.hash.get();
                trace!("hash: {hash}");
                format!("text-[2rem] grid place-items-center h-[5rem] w-[5rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05")
            }
            // style:background-image=move || format!("url(\"{url}\")")
            >"+"</button>
    }
}

// #[component]
// pub fn SVGTrash(#[prop(optional, into)] class: String) -> impl IntoView {
//     view! {
//         <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=class>
//           <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
//         </svg>
//     }
// }

// #[component]
// pub fn SVGArrowDown(#[prop(optional, into)] class: String) -> impl IntoView {
//     view! {
//         <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=class>
//           <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
//         </svg>
//     }
// }

// #[component]
// pub fn SVGTriangle(#[prop(optional, into)] class: String) -> impl IntoView {
//     view! {
//         <svg width="12" height="11" viewBox="0 0 12 11" fill="none" xmlns="http://www.w3.org/2000/svg" class=class>
//             <path d="M6.63067 9.75C6.24577 10.4167 5.28352 10.4167 4.89862 9.75L0.135483 1.5C-0.249417 0.833333 0.231708 -2.83122e-07 1.00151 -2.83122e-07L10.5278 -2.83122e-07C11.2976 -2.83122e-07 11.7787 0.833333 11.3938 1.5L6.63067 9.75Z" fill="currentColor"/>
//         </svg>
//     }
// }
