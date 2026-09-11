use super::{EditSaveCancel, LengthCounter, PostApi};
use crate::hook::Spawner;
use crate::page::create_client;
use crate::{AutoTextArea, Errs, PageState};
use catsquad_shared::MAX_POST_TAGS_LENGTH;
use leptos::html;
use leptos::prelude::*;
use web_sys::HtmlTextAreaElement;

#[component]
pub fn PostTags(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let page = PageState::get();
    let edit_tags_input = NodeRef::<html::Textarea>::new();

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
        let (post_id, Some(tags)) = (
            post_id.get(),
            edit_tags_input
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        spawner.spawn(async move {
            let client = create_client();
            post_api.update_tags(&client, post_id, tags).await;
        });
    };

    let when_is_user = move || page.is_logged_in().unwrap_or_default();
    let when_edit = move || post_api.update_tags_mode.get();
    let when_tag_full = move || post_api.tags.with(|v| !v.is_empty());
    let on_input = move |v: HtmlTextAreaElement| post_api.live_tags_length.set(v.value().len());
    let tags_str = move || post_api.tags.get();

    view! {
        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4">
            <div class="flex justify-between">
                <h1 class="text-[1.3rem] text-base0F">"Tags"</h1>
                <div class="flex gap-2 items-center">

                    <Show when=when_is_user>
                        <Show when=when_edit>
                            <LengthCounter
                                counter_current=post_api.live_tags_length
                                counter_max=MAX_POST_TAGS_LENGTH
                            />
                        </Show>
                        <EditSaveCancel
                            id="tags"
                            when=when_edit
                            on_save=move || edit_tags_save()
                            on_cancel=move || edit_tags()
                            on_edit=move || edit_tags()
                        />
                    </Show>

                </div>

            </div>

            <Errs error=post_api.err_tags />

            <div class="text-ellipsis flex flex-wrap gap-1 overflow-hidden padding max-w-[calc(100vw-1rem)]">

                <Show when=when_edit fallback={move || view!{
                    <Show when=when_tag_full fallback={move || view!{<span class="text-base03">"No tags."</span>} }>
                        { tags }
                    </Show>
                } }>
                    <AutoTextArea
                        id="post_tags_editable"
                        node_ref=edit_tags_input
                        on_input=on_input
                        class="text-[1.1rem] break-all focus:outline-none! appearance-none border-none resize w-full rounded bg-base01 px-4 py-2"
                        min_height=100.0
                    >
                         {tags_str}
                    </AutoTextArea>
                </Show>
             </div>
        </div>
    }
}
