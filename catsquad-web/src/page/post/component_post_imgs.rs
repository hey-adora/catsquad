use crate::page::post::post_api::PostApi;
use leptos::prelude::*;

#[component]
pub fn PostImgs(post_api: PostApi) -> impl IntoView {
    let imgs = move || -> Vec<AnyView> {
        let imgs = post_api.imgs_links.get();
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
                .map(|(i, (url, ratio))| view! {
                    <div style:aspect-ratio=ratio.to_string() class="w-full grid place-items-center bg-base02">
                        <img id=move || format!("id{i}") class="" src=url />
                    </div>
                }.into_any())
                .collect_view()
    };

    view! {

        <div class="hidden lg:flex flex-col gap-2 lg:overflow-y-scroll" >
            { imgs }
        </div>
    }
}
