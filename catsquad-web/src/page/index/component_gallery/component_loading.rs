use leptos::prelude::*;

#[component]
pub fn Loading() -> impl IntoView {
    let img_class = " w-full h-full bg-base02 animate-pulse";

    // let view_fake_imgs = (0..(4 * 1))
    //     .map(|_| view! {<div class=img_class></div>})
    //     .collect_view();

    let view_fake_imgs_sm = (0..(4 * 2))
        .map(|_| view! {<div class=img_class></div>})
        .collect_view();

    let view_fake_imgs_md = (0..(4 * 4))
        .map(|_| view! {<div class=img_class></div>})
        .collect_view();
    // transform -translate-x-1/2 -translate-y-1/2
    view! {
        <div class="hidden md:grid grid-cols-[auto_auto_auto_auto] grid-rows-[auto] gap-[0.5rem] absolute top-0 left-0  h-full w-full overflow-hidden">
            {view_fake_imgs_md}
        </div>

        <div class="grid md:hidden grid-cols-[auto_auto] grid-rows-[auto] gap-[0.5rem] absolute top-0 left-0 h-full w-full overflow-hidden">
            {view_fake_imgs_sm}
        </div>

    }
}
// <div class="grid sm:hidden grid-cols-[auto]                grid-rows-[auto] gap-[0.5rem] absolute top-0 left-0 h-[calc(100%-1rem))] w-[calc(100%-1rem)] overflow-hidden">
//     {view_fake_imgs}
// </div>
