use crate::{
    BtnPrimary, Errs, PageState,
    hook::Spawner,
    page::{
        create_client,
        post::{comments_basic::CommentsBaisc, component_comment::Comment},
    },
};
use catsquad_log::prelude::*;
use catsquad_shared::LINK_WEB_LOGIN;
use leptos::{html, prelude::*};

#[component]
pub fn Comments(#[prop(optional, into)] post_id: Signal<i64>) -> impl IntoView {
    let page = PageState::get();
    let spawner_comments = Spawner::new();
    let comment_container_ref = NodeRef::<html::Div>::new();
    let comment_input_ref = NodeRef::<html::Textarea>::new();
    let comment_basic = CommentsBaisc::new(spawner_comments);
    let post_comment = move || {
        let Some(input_elm) = comment_input_ref.get() else {
            return;
        };
        spawner_comments.spawn(async move {
            let text = input_elm.value();
            let client = create_client();
            comment_basic.comments_manual.post(&client, text).await;
            let post_err = comment_basic.err_post.get_untracked();
            if !post_err.is_empty() {
                return;
            }
            input_elm.set_value("");
        });
    };
    Effect::new(move || {
        trace!("comments basic start");
        let (Some(comment_container_ref), post_id) = (comment_container_ref.get(), post_id.get())
        else {
            return;
        };

        if post_id == 0 {
            return;
        }

        trace!("comments basic observe");
        spawner_comments.spawn(comment_basic.init(comment_container_ref.into(), post_id));
    });

    let when_is_loading = move || page.acc_pending();
    let when_is_guest = move || !page.is_logged_in().unwrap_or_default() && !page.acc_pending();
    let when_is_user = move || page.is_logged_in().unwrap_or_default();

    view! {
        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4 pb-[7rem]">
            <h1 class="text-[1.3rem] text-base0F ">"Comments"</h1>
            <Show when=when_is_loading>
                <div class="bg-base01 rounded-xl grid place-items-center py-5 px-2">
                    <div class="flex flex-col gap-2">
                        <div class="text-base03">"loading..."</div>
                    </div>
                </div>
            </Show>
            <Show when=when_is_guest>
                <div class="bg-base01 rounded-xl grid place-items-center py-5 px-2" >
                    <div class="flex flex-col gap-2">
                        <div class="text-base03">"You must login to comment"</div>
                        <a class="mx-auto rounded-full font-semibold text-[1rem] font-medium px-[0.8rem] py-[0.2rem] hover:bg-base05 bg-base0D text-base01" href=LINK_WEB_LOGIN >"Login"</a>
                    </div>
                </div>
            </Show>
            <Show when=when_is_user>
                <div class="flex bg-base01 rounded-xl flex-col gap-1 py-2 px-4" >
                    <textarea placeholder="Comment" node_ref=comment_input_ref class="focus:outline-none! appearance-none border-none resize text-[1.1rem]" id="story" name="story" rows="3" cols="5" ></textarea>
                    <Errs error=comment_basic.err_post />
                    <div class="flex justify-between place-items-center">
                        <p class="text-[1rem]">"0/2000"</p>
                        <BtnPrimary id=move|_:()|String::new() on_click=move |_| post_comment() class=move || "ml-auto">
                            "Post"
                        </BtnPrimary>
                    </div>
                </div>
            </Show>

            <div class="flex flex-col gap-2">
                <div node_ref=comment_container_ref class=" flex flex-col gap-2 relative 0h-[20rem] 0overflow-y-scroll">
                    <For
                        each=move || comment_basic.items.get()
                        key=|state| state.id.clone()
                        let(data)
                    >
                        {
                            view!{
                                <Comment
                                    parent_id=0
                                    parent_items=comment_basic.items
                                    parent_reply_count=comment_basic.replies_count
                                    comment=data
                                    post_id
                                    max_depth=2
                                    parent_depth=0 />
                            }.into_any()
                        }
                    </For>
                </div>
                <Show when=move || comment_basic.comments_manual.err_fetch.with(|v| !v.is_empty()) >
                    <ul class="ml-[1rem] text-base08 list-disc">
                        {move || comment_basic.comments_manual.err_fetch.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                    </ul>
                </Show>
            </div>
        </div>
    }
}
