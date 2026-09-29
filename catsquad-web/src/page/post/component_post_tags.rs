use super::PostApi;
use crate::hook::Spawner;
use crate::page::create_client;
use crate::{PageState, TextEditor};
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
                    <div class="bg-base02 rounded-lg text-[1rem] px-3 py-1 break-all max-w-full ">
                        {v}
                    </div>

                }
            })
            .collect_view()
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

    let when_is_owner = move || page.acc_username() == post_api.author_username.get();
    let when_tag_full = move || post_api.tags.with(|v| !v.is_empty());

    view! {
        <TextEditor
            id_prefix="tags"
            title="Tags"
            text=post_api.tags
            text_length=post_api.live_tags_length
            is_owned=when_is_owner
            edit_mode_enabled=post_api.update_tags_mode
            max_length=MAX_POST_TAGS_LENGTH
            errors=post_api.err_tags
            on_save=edit_tags_save
            node_ref=edit_tags_input
            >
                <div class="flex flex-wrap gap-1">
                    <Show when=when_tag_full fallback={move || view!{<span class="text-base03">"No tags."</span>} }>
                        { tags }
                    </Show>
                </div>
        </TextEditor>
    }
}
