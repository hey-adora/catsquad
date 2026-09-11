use crate::{
    BtnDelete, PageState, SVGTrash,
    hook::Spawner,
    page::{
        create_client,
        post::{post_api::PostApi, post_like_state::PostLikeState},
    },
};
use catsquad_client::XMLSender;
use catsquad_web_utils::time::micro_to_str;
use leptos::prelude::*;

#[component]
pub fn PostStats(
    // post_like: PostLikeState<XMLSender>,
    // spawner: Spawner,
    post_api: PostApi,
    // #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let page = PageState::get();
    // XMLSender
    // create_client()
    // let post_like = PostLikeState::new(create_client());

    // Effect::new();
    // let delete_post = move |_| {
    //     let post_id = post_id.get();
    //     if post_id == 0 {
    //         return;
    //     }
    //     spawner.spawn(async move {
    //         let client = create_client();
    //         post_api.delete(&client, post_id).await;
    //     });
    // };
    let created_at =
        move || micro_to_str(page.get_time().saturating_sub(post_api.created_at.get()));
    let like_count = move || post_api.likes.get();
    // let l

    view! {
        <div class="col-span-2 flex justify-between ">
            <div>
                <p class="text-base03">{created_at}" ago"</p>
            </div>
            <div>
                <p>{like_count}" favorites"</p>
            </div>
        </div>
    }
}
// <SVGTrash class="size-[1rem] text-base08 "/>
