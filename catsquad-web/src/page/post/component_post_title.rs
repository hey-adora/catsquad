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
    let when_edit_on = move || post_api.update_title_mode.get();
    let when_edit_off = move || !when_edit_on();
    let class_title = move || {
        format!(
            "text-[1.5rem] break-all {}",
            if title_is_empty() {
                "text-base03"
            } else {
                "text-base0F"
            }
        )
    };
    let class_on_edit = move || {
        format!(
            " {} ",
            if when_edit_on() {
                "flex gap-2 place-items-center"
            } else {
                "inline-block"
            }
        )
    };
    let on_input = move |v: HtmlTextAreaElement| post_api.live_title_length.set(v.value().len());

    view! {
        <div class="flex flex-col gap-2">

            <h1 >
                <Show when=when_edit_off>
                    <span class=class_title>{ title }</span>
                </Show>

                "  "

                <Show when=when_is_user >
                    <span class=class_on_edit>
                        <Show when=move|| post_api.update_title_mode.get()>
                            <LengthCounter
                                class="ml-auto "
                                counter_current=move||post_api.live_title_length.get()
                                counter_max=MAX_POST_TITLE_LENGTH
                            />
                        </Show>

                        <EditSaveCancel
                            id="title"
                            class_edit="ml-auto"
                            when=when_edit_on
                            on_save=move || edit_title_save()
                            on_cancel=move || edit_title_mode_toggle()
                            on_edit=move || edit_title_mode_toggle()
                        />
                    </span>
                </Show>
            </h1>

            <Errs
                error=move||post_api.err_title.get()
            />


        </div>

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
    }
}
