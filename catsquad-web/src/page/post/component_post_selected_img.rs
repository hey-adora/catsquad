use crate::page::post::post_api::PostApi;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn PostSelectedImg(post_api: PostApi) -> impl IntoView {
    let location = use_location();
    let selected_img = move || -> AnyView {
        let hash = location.hash.get();
        let imgs_links = post_api.imgs_links.get();
        let selected_n = if hash.len() > 3 {
            usize::from_str_radix(&hash[3..], 10).unwrap_or_default()
        } else {
            0
        };
        let Some((selected_url, selected_ratio)) = imgs_links.get(selected_n).cloned() else {
            return view! {
                <p>
                    "No Image"
                </p>
                // <img id=move || format!("id0") class="max-h-full" src=selected_url />
            }
            .into_any();
        };

        view! {
            <img id=move || format!("id{selected_n}") class="max-h-full" src=selected_url />
        }
        .into_any()
    };

    view! {

        <div class="lg:hidden h-[50vh] flex justify-center place-items-center bg-base02" >
            { selected_img }
        </div>
    }
}
