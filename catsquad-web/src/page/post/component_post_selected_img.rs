use crate::{hook::PostImageId, page::post::post_api::PostApi};
use catsquad_shared::link_relative_post_image_bytes_get_by_hash;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn PostSelectedImg(#[prop(into)] post_id: Signal<i64>, post_api: PostApi) -> impl IntoView {
    let location = use_location();
    let selected_img = move || -> AnyView {
        let post_id = post_id.get();
        let hash = location.hash.get();
        let img_id = PostImageId::from(hash);
        let hash = img_id.0;
        let img_id_str = img_id.to_id();
        let selected_file = post_api.imgs.with(|imgs| {
            imgs.iter()
                .find(|v| v.with_untracked(|v| v.hash == img_id.0))
                .cloned()
        });

        let Some(_) = selected_file else {
            return view! {
                <p>
                    "No Image"
                </p>
            }
            .into_any();
        };

        let url = link_relative_post_image_bytes_get_by_hash(post_id, hash);

        view! {
            <img id=img_id_str class="max-h-full" src=url />
        }
        .into_any()
    };

    view! {
        <div class="lg:hidden h-[50vh] flex justify-center place-items-center bg-base02" >
            { selected_img }
        </div>
    }
}
