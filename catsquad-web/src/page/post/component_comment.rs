use crate::hook::Spawner;
use crate::page::create_client;
use crate::page::post::comments_api::{CommentKind, CommentsApi};
use crate::{PageState, SVGArrowDown, SVGTrash, SVGTriangle, hook::EventListener};
use catsquad_log::prelude::*;
use catsquad_shared::CommentRes;
use catsquad_web_utils::time::{micro_to_str, time_now_micro};
use catsquad_web_utils::timeout::set_timeout_fn;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use std::time::Duration;
use web_sys::{HtmlDivElement, ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition};

#[component]
pub fn Comment(
    parent_id: i64,
    parent_items: RwSignal<Vec<CommentRes>, LocalStorage>,
    parent_reply_count: RwSignal<u32, LocalStorage>,
    comment: CommentRes,
    post_id: Signal<i64>,
    max_depth: usize,
    parent_depth: usize,
) -> impl IntoView {
    let current_depth = parent_depth + 1;
    let global_state = PageState::get();
    let comment_container_ref = NodeRef::<html::Div>::new();
    let comment_edit_ref = NodeRef::new();
    let comment_input_ref = NodeRef::<html::Textarea>::new();
    let flatten = current_depth >= max_depth;
    let reply_render_comments = current_depth <= max_depth;
    // let reply_btn_shown = RwSignal::new(false);
    let replies_shown = RwSignal::new(false);
    // let edit_enabled = RwSignal::new(false);
    // let api = ApiWeb::new();
    let comment_key = comment.id.clone();
    let is_owned_fn = {
        let user_username = comment.user_username.clone();
        move || global_state.acc_username() == user_username
    };

    let comment_edit_event = EventListener::new(ev::change, |a| {
        trace!("omg is it working edit magic");

        //
    });
    Effect::new(move || {
        let Some(elm) = comment_edit_ref.get() else {
            return;
        };
        comment_edit_event.add(elm);
    });

    let spawner = Spawner::new();
    let kind = if current_depth < max_depth {
        CommentKind::Reply {
            parent_id: parent_id,
            parent_items: parent_items,
            parent_replies_count: parent_reply_count,
            comment: comment.clone(),
        }
    } else if current_depth == max_depth {
        CommentKind::Flat {
            parent_id: parent_id,
            parent_items: parent_items,
            parent_replies_count: parent_reply_count,
            comment: comment.clone(),
        }
    } else {
        CommentKind::None {
            parent_id: parent_id,
            parent_items: parent_items,
            parent_replies_count: parent_reply_count,
            comment: comment.clone(),
        }
    };

    let comments_manual = CommentsApi::new(10, kind.clone());
    let post_comment = move |_| {
        let Some(input_elm) = comment_input_ref.get() else {
            return;
        };
        spawner.spawn(async move {
            let text = input_elm.value();
            let client = create_client();
            comments_manual.post(&client, text).await;
            let post_err = comments_manual.err_post.get_untracked();
            if !post_err.is_empty() {
                return;
            }
            input_elm.set_value("");
        });
    };
    let delete_comment = move |_| {
        spawner.spawn(async move {
            let client = create_client();
            comments_manual.delete(&client).await;
        });
    };
    let fetch_comments = move || {
        spawner.spawn(async move {
            let time = time_now_micro();
            let client = create_client();
            comments_manual.fetch(time, &client).await;
        });
    };
    let toggle_btn = move |_| {
        comments_manual.show_editor.update(|v| *v = !*v);
        let show = comments_manual.show_editor.get_untracked();
        if !show {
            return;
        }
        replies_shown.set(true);
        spawner.spawn(async move {
            let time = time_now_micro();
            let client = create_client();
            comments_manual.fetch(time, &client).await;
        });
    };
    let toggle_replies = move |_| {
        trace!("KILL ME YOU FUCK");
        replies_shown.update(|v| *v = !*v);
        let show = replies_shown.get_untracked();
        trace!("KILL ME YOU FUCK 2 {show}");
        if !show {
            return;
        }
        trace!("KILL ME YOU FUCK 3 {show}");
        spawner.spawn(async move {
            let time = time_now_micro();
            let client = create_client();
            comments_manual.fetch(time, &client).await;
        });
    };

    Effect::new(move || {
        trace!("comments manual start");
        let post_id = post_id.get();
        if post_id == 0 {
            return;
        }

        trace!("comments manual observe depth({current_depth})");
        comments_manual.init(post_id);
    });

    let show_replies_fn = move || {
        (reply_render_comments && replies_shown.get())
            || (reply_render_comments && comments_manual.show_editor.get())
    };
    let show_line = move || {
        current_depth > 0 && comments_manual.items.with(|v| v.len() > 0) && show_replies_fn()
    };

    let is_bubble = 'f: {
        if !kind.is_none() {
            break 'f false;
        }
        let Some(last) = comment.parent_id.last() else {
            break 'f false;
        };
        *last != parent_id
    };

    let bubble = 'f: {
        if !is_bubble {
            break 'f None;
        }
        // let last = comment.parent_key.last().cloned();
        let Some(last) = comment.parent_id.last() else {
            break 'f None;
        };
        parent_items.with(|v| v.iter().find(|v| v.id == *last).cloned())
    };

    let on_bubble_click = {
        let bubble = bubble.clone();
        let comment_key = comment_key.clone();
        move || {
            let Some(elm) = bubble.and_then(|v| document().get_element_by_id(&v.id.to_string()))
            else {
                warn!("cant find element for bubble click {}", comment_key);
                return;
            };
            let options = ScrollIntoViewOptions::new();
            options.set_behavior(ScrollBehavior::Auto);
            options.set_block(ScrollLogicalPosition::Center);
            options.set_inline(ScrollLogicalPosition::Center);
            elm.scroll_into_view_with_scroll_into_view_options(&options);
            let anim = "animate-[glow_1s_linear]";
            let classes = elm.class_list();
            let _ = classes.add_1(anim);
            // elm.set_class_name(anim);
            // set_timeout(cb, duration);
            let result = set_timeout_fn(
                move || {
                    let _ = classes.remove_1(anim);
                },
                Duration::from_secs(1),
            );
            if let Err(err) = result {
                error!("{err}");
            }
        }
    };
    let on_bubble_click_fn = move |_| {
        (on_bubble_click.clone())();
    };

    // micro_to_str(ns)

    let click_edit = move |_| {
        // edit_enabled.update(|v| *v = !*v);
        if comments_manual.edit_mode.get_untracked() {
            let Some(text) = comment_edit_ref
                .get_untracked()
                .and_then(|v: HtmlDivElement| v.text_content())
            else {
                return;
            };
            spawner.spawn(async move {
                let client = create_client();
                comments_manual.update_comment(&client, text).await;
            });
        }

        comments_manual.edit_mode.set(true);
    };

    let click_cancel = move |_| {
        let Some(elm) = comment_edit_ref.get_untracked() as Option<HtmlDivElement> else {
            return;
        };

        let txt = comments_manual.text.get_untracked();

        elm.set_text_content(Some(&txt));

        comments_manual.err_update.update(|v| v.clear());
        comments_manual.edit_mode.set(false);
    };

    view! {
        <div class=" flex flex-col "  >
            <div id=comment.id.clone() class=" rounded 0bg-base03 flex flex-col">
                <Show when=move || is_bubble>
                    <button on:click=on_bubble_click_fn.clone() class="cursor-pointer flex gap-2 items-center">
                        <div class="flex place-items-end h-[1.5rem] w-[3.2rem] shrink-0">
                            <div class=" mb-[0.5rem] w-[1.7rem] h-[0.5rem] border-base05 border-l-[0.2rem] border-t-[0.2rem] rounded-tl-[2rem] ml-auto box-border shrink-0"></div>
                        </div>
                        <p class="ml-2 text-[1rem] rounded-full h-[1rem] w-[1rem] shrink-0 bg-base05"></p>
                        <div>
                            {bubble.clone().map(|v| v.text).unwrap_or_else(|| "failed to load msg".to_string())}
                        </div>
                    </button>
                </Show>
                <div class="grid grid-cols-[auto_1fr] grid-rows-[100%] ">
                    <div class=" mb-[0.5rem] w-[3.2rem] h-full grid grid-rows-[auto_100%] items-start place-items-center shrink-0">
                        <p class="text-[1rem] rounded-full h-[3.2rem] w-[3.2rem] shrink-0 bg-base05"></p>
                        <Show when={move || { !comments_manual.is_last() } || show_replies_fn() }>
                            <div class="rounded w-[0.2rem] mt-[0.5rem] h-[calc(100%-3.2rem-0.5rem)] bg-base05 shrink-0"></div>
                        </Show>
                    </div>
                    <div  class="pl-4  flex flex-col w-full group">
                        <div class="flex gap-2 place-items-center ">
                            <div class="text-[1.2rem]"> {comment.user_username.clone()} </div>
                            <div class="text-[1rem] text-base03"> {move || micro_to_str(global_state.get_time().saturating_sub(comment.created_at))}" ago"</div>

                            <Show when={move || is_owned_fn() || comments_manual.edit_mode.get()} >
                                <div class=move || format!(" gap-2 ml-auto place-items-center {}", if comments_manual.edit_mode.get() {"flex"} else {"group-hover:flex hidden"} )>
                                    <button on:click=click_edit class=move || format!("text-center   rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] w-[4rem]  {}", if comments_manual.edit_mode.get() { " hover:bg-base05 bg-base0D text-base01" } else { " text-base05 bg-base01 hover:bg-base05 hover:text-base01" })>
                                        <Show when={move || comments_manual.edit_mode.get() } fallback={move || "Edit" }>
                                            "Save"
                                        </Show>
                                    </button>
                                    <Show when=move || comments_manual.edit_mode.get() >
                                        <button on:click=click_cancel class=move || format!("text-center  rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] w-[4rem] text-base05 bg-base01 hover:bg-base05 hover:text-base01")>
                                            "Cancel"
                                        </button>
                                    </Show>
                                    <Show when=move || !comments_manual.edit_mode.get() >
                                        <button on:click=delete_comment class="">
                                            <SVGTrash class="size-[1.1rem] text-base08 "/>
                                        </button>
                                    </Show>
                                </div>
                            </Show>
                        </div>


                        <div contenteditable={move || comments_manual.edit_mode.get()}
                             node_ref=comment_edit_ref
                             class={move || format!(" text-[1.1rem] break-all focus:outline-none! appearance-none border-none resize w-full rounded {}", if comments_manual.edit_mode.get() { "bg-base01 px-4 py-2" } else { "" })} >
                            {
                                move || comments_manual.text.get()
                            }
                        </div>
                        <Show when=move || comments_manual.err_update.with(|v| !v.is_empty()) >
                            <ul class="ml-[1rem] text-base08 list-disc">
                                {move || comments_manual.err_update.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                            </ul>
                        </Show>
                        <Show when=move || comments_manual.err_delete.with(|v| !v.is_empty()) >
                            <ul class="ml-[1rem] text-base08 list-disc">
                                {move || comments_manual.err_delete.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                            </ul>
                        </Show>
                        // <div class=" mb-2 text-[1.1rem] break-all"> {comment.text} </div>
                        <div class=" h-[1.6rem] flex gap-2 place-items-center">
                            <Show when=move || reply_render_comments >
                                // <button on:click=toggle_replies type="submit" class=move || format!("group  gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] py-[0.2rem]  {}", if replies_shown.get() { "text-base05 bg-base01 hover:bg-base03" } else { "text-base05 bg-base01 hover:bg-base05 hover:text-base01" })>
                                <Show when=move || {comments_manual.replies_count.get() > 0} fallback=move || view!{
                                    <p class=move || format!("group text-base03 gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium ")>
                                        <div class="0group-hover:bg-base01 size-3 bg-base03 aspect-square rounded mx-auto"/>
                                        "no replies"
                                    </p>

                                }>
                                    <button on:click=toggle_replies class=move || format!("group  gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium ")>
                                        <Show when=move || replies_shown.get() fallback={|| view!{
                                            <div class="0group-hover:bg-base01 size-3 bg-base05 aspect-square rounded mx-auto"/>
                                        }}>
                                            <SVGTriangle class="size-3 mx-auto"/>
                                        </Show>
                                        <Show when=move || !comments_manual.kind.with_value(|v| v.is_flat()) fallback={|| "replies"}>
                                            {move || comments_manual.replies_count.get() }
                                            " replies"
                                        </Show>
                                    </button>
                                </Show>
                            </Show>
                            <Show when=move || global_state.is_logged_in().unwrap_or_default()>
                                <button on:click=toggle_btn type="submit" class=move || format!("  rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] w-[4rem]  {}", if comments_manual.show_editor.get() { "text-base05 bg-base01 hover:bg-base03" } else { "text-base05 bg-base01 hover:bg-base05 hover:text-base01" })>
                                    <Show when=move || comments_manual.show_editor.get() fallback=|| "Reply">
                                        <SVGArrowDown class="size-4 mx-auto"/>
                                    </Show>
                                </button>
                            </Show>
                        </div>
                        <Show when=move || comments_manual.show_editor.get()>
                            <div class=move || format!("flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 w-full {}", if global_state.is_logged_in().unwrap_or_default() || !global_state.acc_pending() { "" } else { "hidden" })  >
                                <textarea placeholder="Comment" node_ref=comment_input_ref class="focus:outline-none! appearance-none border-none resize text-[1.1rem] w-full" rows="2" wrap="hard"  ></textarea>
                                // <ul class="text-base08 list-disc ml-[1rem]">
                                //     {move || post_comments.err_post.get().map(|v| v.trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view()) }
                                // </ul>on:submit=post_comment
                                <ul class="text-base08 list-disc ml-[1rem]">
                                    {move || comments_manual.err_post.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                                <div class="flex justify-between place-items-center">
                                    <p>"0/2000"</p>
                                    <button on:click=post_comment class="ml-auto rounded-full font-medium text-[0.8rem] font-bold px-[0.8rem] py-[0.2rem] hover:bg-base0D bg-base03 text-base05 text-center w-[5rem]">
                                        "Reply"
                                    </button>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
            <div class=move || format!("grid grid-rows-[100%] grid-cols-[auto_1fr] w-full {}", if reply_render_comments && show_replies_fn() {"pt-2"} else {""})>
                <Show when=show_line>
                    <div class="relative ml-[1.5rem] w-[1rem] h-full flex justify-sart shrink-0">
                        <div class="w-[1rem] h-[1.61rem] border-base05 border-l-[0.2rem] border-b-[0.2rem] rounded-bl-[2rem] ml-auto box-border shrink-0"></div>
                        <Show when=move || { !comments_manual.is_last() }>
                            <div class="absolute w-[0.2rem] h-full bg-base05 shrink-0"></div>
                        </Show>
                    </div>
                </Show>
                // <form class=move || format!("mb-4 flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 w-full {}", if global_state.is_logged_in().unwrap_or_default() || !global_state.acc_pending() { "" } else { "hidden" }) on:submit=post_comments.on_comment.to_fn() >

                <div class="flex flex-col w-full">
                    // <Show when=move || reply_render_comments && (replies_shown.get() || comments_manual.reply_editor_show.get())>
                    <Show when=move || reply_render_comments>
                        <div node_ref=comment_container_ref class=move || format!("flex flex-col gap-2 0h-[20rem] 0overflow-y-scroll {} ", if show_replies_fn() {""} else {"hidden"} )>
                            {
                                let comment_key = comment_key.clone();

                                view! {
                                    <For
                                        each=move || comments_manual.items.get()
                                        key=|state| state.id.clone()
                                        let(data)
                                    >
                                        {
                                            // let key = data.key.clone();
                                            // let is_last = comments_manual.items.with(|v| v.last().map(|v| v.key == key).unwrap_or_default());
                                            let comment_key = comment_key.clone();
                                            view!{
                                                <Comment
                                                    parent_id=comment_key.clone()
                                                    parent_items=comments_manual.items
                                                    parent_reply_count=comments_manual.replies_count
                                                    comment=data
                                                    post_id
                                                    max_depth=max_depth
                                                    parent_depth=current_depth />
                                            }.into_any()
                                        }
                                    </For>
                                }
                            }
                            <button on:click=move |_| { fetch_comments(); } class=move || format!("px-4 py-2 bg-base01 rounded-xl text-center text-base05 font-[1.2rem] w-full {}", if comments_manual.finished.get() {"hidden"} else {""})>
                                "load more"
                            </button>
                            <Show when=move || comments_manual.err_fetch.with(|v| !v.is_empty()) >
                                <ul class="text-base08 list-disc">
                                    {move || comments_manual.err_fetch.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                            </Show>
                        </div>
                        // { post_comment_views }
                    </Show>
                </div>

            </div>
        </div>
    }
}
