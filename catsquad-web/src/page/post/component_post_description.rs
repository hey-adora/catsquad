use super::{EditSaveCancel, LengthCounter};
use crate::{
    AutoTextArea, Errs, PageState,
    hook::Spawner,
    page::{create_client, post::post_api::PostApi},
};
use catsquad_log::prelude::*;
use catsquad_shared::MAX_POST_DESCRIPTION_LENGTH;
use leptos::{html, prelude::*};
use web_sys::HtmlTextAreaElement;

#[component]
pub fn PostDescription(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let page = PageState::get();
    let description_input_editor = NodeRef::<html::Textarea>::new();
    let when_is_user = move || page.is_logged_in().unwrap_or_default();
    let when_edit = move || post_api.update_description_mode.get();

    let edit_description_mode_toggle = move || {
        let description_len = post_api.description.with_untracked(|v| v.len());
        post_api.live_description_length.set(description_len);
        post_api.update_description_mode.update(|v| *v = !*v);
    };

    let edit_description_save = move || {
        let (post_id, Some(new_description)) = (
            post_id.get(),
            description_input_editor
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        spawner.spawn(async move {
            let client = create_client();
            post_api
                .update_description(&client, post_id, new_description)
                .await;
        });
    };

    let description = move || post_api.description.get();
    let on_input =
        move |v: HtmlTextAreaElement| post_api.live_description_length.set(v.value().len());
    let class_description = move || {
        format!(
            "whitespace-break-spaces break-all text-ellipsis overflow-hidden padding max-w-[calc(100vw-1rem)] rounded {}",
            if post_api.description.with(|v| v.is_empty()) {
                "text-base03"
            } else {
                ""
            }
        )
    };

    view! {

        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4">
            <div class="flex justify-between">
                <h1 class="text-[1.3rem] text-base0F">"Description"</h1>
                <div class="flex gap-2 items-center">

                    <Show when=when_is_user >
                        <Show when=when_edit>
                            <LengthCounter
                                counter_current=move||post_api.live_description_length.get()
                                counter_max=move||MAX_POST_DESCRIPTION_LENGTH
                            />
                        </Show>
                        <EditSaveCancel
                            id="description"
                            when=when_edit
                            on_save=move || edit_description_save()
                            on_cancel=move || edit_description_mode_toggle()
                            on_edit=move || edit_description_mode_toggle()
                        />
                    </Show>
                </div>
            </div>

            <Errs error=post_api.err_description />

            <Show when=move || post_api.update_description_mode.get() fallback=move || view!{
                <pre
                    id="post_description"
                    class=class_description>
                    { description }
                </pre>
            }>

            <AutoTextArea
                id="post_description_editable"
                node_ref=description_input_editor
                on_input=on_input
                class="bg-base01 text-base05 px-4 py-2 rounded"
            >
                { description }
            </AutoTextArea>
            </Show>
        </div>
    }
}
