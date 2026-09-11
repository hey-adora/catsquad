use crate::{hook::Spawner, page::post::post_api::PostApi};
use catsquad_log::prelude::*;
use catsquad_web_utils::file::GetFiles;
use leptos::{html, prelude::*};
use leptos_router::hooks::use_location;
use web_sys::HtmlInputElement;

#[component]
pub fn PostPreviews(
    spawner: Spawner,
    post_api: PostApi,
    #[prop(optional, into)] post_id: Signal<i64>,
) -> impl IntoView {
    let upload_image = NodeRef::<html::Input>::new();

    let on_upload = move |_| {
        let (Some(files),): (Option<Vec<web_sys::File>>,) = (
            (upload_image.get_untracked())
                .and_then(|f: HtmlInputElement| f.files())
                .map(|f| f.get_files()),
            // upload_title.get_untracked() as Option<HtmlInputElement>,
            // upload_description.get_untracked() as Option<HtmlTextAreaElement>,
            // upload_tags.get_untracked() as Option<HtmlTextAreaElement>,
        ) else {
            return;
        };

        // uploader.upload(&files[..]);

        trace!("files selected: {}", files.len());
    };

    let previews = move || {
        let mut imgs = post_api.imgs_links.get();

        // imgs.push((String::new(), 0.0);

        let mut views = imgs
            .into_iter()
            .enumerate()
            .map(|(i, (url, ratio))| {
                view! {
                    <PreviewImg
                        index=move|| i
                        link=move|| url.clone()
                    />
                }
                .into_any()
            })
            .collect_view();

        let preview_add = {
            // let i = views.len();
            // let id = format!("#id{i}");
            // let id2 = id.clone();

            view! {
                <div>
                    <label
                        id="previw_add"
                        for="image"
                        class="text-[2rem] grid place-items-center h-[5rem] w-[5rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05"
                        >"+"</label>
                    <input class="left-0 top-0 w-0 h-0 absolute z-[-1] opacity-0" on:change=on_upload type="file" id="image" name="image" node_ref=upload_image multiple />
                </div>
            }
        };

        views.push(preview_add.into_any());

        views
    };

    view! {
        <div class="flex justify-start gap-2 flex flex-wrap">
            { previews }
        </div>
    }
}

#[component]
pub fn PreviewImg(
    #[prop(optional, into)] index: Option<Callback<(), usize>>,
    #[prop(optional, into)] link: Option<Callback<(), String>>,
    // #[prop(optional, into)] hash: Option<Callback<(), String>>,
) -> impl IntoView {
    let location = use_location();

    let index_fn = move || index.map(|v| v.run(())).unwrap_or_default();
    let link_fn = move || link.map(|v| v.run(())).unwrap_or_default();
    let id_fn = move || format!("#id{}", index_fn());
    let is_selected = move || {
        let hash = location.hash.get();
        let index = index_fn();
        let id = id_fn();
        id == hash || (hash.is_empty() && index == 0)
    };

    view! {
        <a
            href=id_fn
            class=move || format!("h-[5rem] w-[5rem] bg-base02 bg-cover bg-center rounded-xl {}", if is_selected() {"border-3 border-base05"} else {""})
            style:background-image=move || format!("url(\"{}\")", link_fn())
        >
        </a>
    }
}
