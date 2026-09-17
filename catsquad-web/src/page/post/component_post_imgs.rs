use crate::{
    hook::{ParsedPostImage, PostImageId},
    page::post::post_api::PostApi,
};
use leptos::prelude::*;

#[component]
pub fn PostImgs(#[prop(optional, into)] post_id: Signal<i64>, post_api: PostApi) -> impl IntoView {
    let imgs = move || -> Vec<AnyView> {
        let post_id = post_id.get();
        let imgs = post_api.imgs_links(post_id);
        if imgs.is_empty() {
            let view = view! {
                    <div style:aspect-ratio="1.0" class=" w-full h-[50%] grid place-items-center bg-base02">
                        <p id="id0" >"No Images"</p>
                        // <img id=move || format!("id{i}") class="" src=url />
                    </div>

            }
            .into_any();
            return vec![view];
        }
        imgs
                .into_iter()
                .enumerate()
                .map(|(i, (url, ratio, hash))| {
                    let id = PostImageId::new(hash).to_id();
                    view! {
                        <div style:aspect-ratio=ratio.to_string() class="w-full grid place-items-center bg-base02">
                            <img id=id class="" src=url />
                        </div>
                    }.into_any()
                })
                .collect_view()
    };

    view! {

        <div class="hidden lg:flex flex-col gap-2 lg:overflow-y-scroll" >
            { imgs }
        </div>
    }
}
