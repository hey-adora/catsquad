use crate::{
    BtnDelete, SVGTrash,
    hook::Spawner,
    page::{create_client, post::post_api::PostApi},
};
use catsquad_shared::PostState;
use leptos::prelude::*;

#[component]
pub fn PostControls(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let delete_post = move |_| {
        let post_id = post_id.get();
        if post_id == 0 {
            return;
        }
        spawner.spawn(async move {
            let client = create_client();
            post_api.delete(&client, post_id).await;
        });
    };

    let state = move || match post_api.post_state.get() {
        Some(PostState::Draft) => "Draft",
        Some(PostState::Active) => "Public",
        Some(PostState::Hidden) => "Hidden",
        None => "",
    };

    let state_color = move || match post_api.post_state.get() {
        Some(PostState::Draft) => "bg-base03",
        Some(PostState::Active) => "bg-base0B text-base01",
        Some(PostState::Hidden) => "bg-base08 text-base01",
        None => "",
    };

    let class_state = move || format!("{} font-bold px-2 py-1 rounded-md", state_color());

    view! {
        <div class="col-span-2 flex justify-between px-4 md:px-6 ">
            <div>
                <p class=class_state>{state}</p>
            </div>
            <div>
                <BtnDelete on_click=delete_post>
                    "Delete"
                </BtnDelete>
            </div>
        </div>
    }
}
// <SVGTrash class="size-[1rem] text-base08 "/>
