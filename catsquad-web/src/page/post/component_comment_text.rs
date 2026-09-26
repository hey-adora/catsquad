use crate::AutoTextArea;
use crate::Errs;
use crate::page::post::comments_api::CommentsApi;
use leptos::{html, prelude::*};

// #[prop(optional, into)] when: Signal<bool>,

#[component]
pub fn CommentText(
    comments_manual: CommentsApi,
    textarea_input: NodeRef<html::Textarea>,
) -> impl IntoView {
    // html::Textarea
    let edit_mode = move || comments_manual.edit_mode.get();
    let normal_mode = move || !edit_mode();
    let comment_text = move || comments_manual.text.get();

    // on_input=textarea_input
    // class={move || format!(" text-[1.1rem] break-all focus:outline-none! appearance-none border-none resize w-full rounded whitespace-pre-wrap {}", if comments_manual.edit_mode.get() { "bg-base01 px-4 py-2" } else { "" })} >

    view! {
        <Show when=normal_mode>
            <pre
                 class=" text-[1.0rem] break-all focus:outline-none! appearance-none border-none resize w-full rounded whitespace-pre-wrap " >
                { comment_text }
            </pre>
        </Show>
        <Show when=edit_mode>
            <AutoTextArea
                id="post_description_editable"
                node_ref=textarea_input
                class="bg-base01 text-base05 px-4 py-2 rounded"
                min_height=100.0
            >
                { comment_text }
            </AutoTextArea>
        </Show>
        <Errs error=comments_manual.err_update />
        <Errs error=comments_manual.err_delete />
    }
}
