use super::component_length_counter::LengthCounter;
use crate::{
    AutoTextArea, Errs, PageState,
    hook::Spawner,
    page::{
        create_client,
        post::{component_edit_btn::EditSaveCancel, post_api::PostApi},
    },
};
use catsquad_log::prelude::*;
use catsquad_shared::MAX_POST_TITLE_LENGTH;
use leptos::{html, prelude::*};
use web_sys::HtmlTextAreaElement;

#[component]
pub fn PostTitle(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let page = PageState::get();
    let input_title = NodeRef::<html::Textarea>::new();

    let title = move || {
        let mut title = post_api.title.get();

        if title.is_empty() {
            title.push_str("No Title.");
        }

        title
    };
    let title_is_empty = move || post_api.title.with(|v| v.is_empty());
    let edit_title_mode_toggle = move || {
        trace!("toggling title edit mode");
        post_api.update_title_mode.update(|v| *v = !*v);
    };

    let edit_title_save = move || {
        let (post_id, Some(title)) = (
            post_id.get(),
            input_title
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        if post_id == 0 {
            error!("post_id not set");
            return;
        }
        trace!("{title}");
        spawner.spawn(async move {
            let client = create_client();
            post_api.update_title(&client, post_id, title).await;
        });
    };

    let when_is_user = move || page.is_logged_in().unwrap_or_default();
    let when_edit_btn = move || post_api.update_title_mode.get();
    let class_title = move || {
        format!(
            "text-[1.5rem] text-ellipsis {}",
            if title_is_empty() {
                "text-base03"
            } else {
                "text-base0F"
            }
        )
    };
    let on_input = move |v: HtmlTextAreaElement| post_api.live_title_length.set(v.value().len());

    view! {
        <Show when=when_is_user >
            <div class="flex gap-2 place-items-center">
                <Errs
                    error=move||post_api.err_title.get()
                />
                <Show when=move|| post_api.update_title_mode.get()>
                    <LengthCounter
                        class="ml-auto"
                        counter_current=move||post_api.live_title_length.get()
                        counter_max=MAX_POST_TITLE_LENGTH
                    />
                </Show>
                <EditSaveCancel
                    id="title"
                    class_edit="ml-auto"
                    when=when_edit_btn
                    on_save=move || edit_title_save()
                    on_cancel=move || edit_title_mode_toggle()
                    on_edit=move || edit_title_mode_toggle()
                />
            </div>
        </Show>
        <div class="flex justify-between">
            <Show when=move || !post_api.update_title_mode.get()>
                <h1 class=class_title >{ title }</h1>
            </Show>
            <Show when=move || post_api.update_title_mode.get()>
                <AutoTextArea
                    id="post_description_editable"
                    placeholder="title"
                    node_ref=input_title
                    on_input
                    min_height=50.0
                    class="w-full bg-base01 text-[1.5rem] text-base05 px-4 py-2 rounded-xl"
                >
                    {title}
                </AutoTextArea>
            </Show>
        </div>
    }
}
