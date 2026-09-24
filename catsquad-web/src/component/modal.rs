use leptos::{html, prelude::*};
use web_sys::MouseEvent;

#[component]
pub fn Modal(
    // #[prop(optional, into)] id: Signal<String>,
    // #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] link: Signal<String>,
    children: Children,
) -> impl IntoView {
    // let id_fn = move || id.get();
    // let class_fn = move || class.get();
    let link_fn = move || link.get();

    view! {
        <div id="modal_global" class="z-[99] bg-base01/80 absolute left-0 top-0 w-[100dvw] h-[100dvh] grid place-content-center">
            <a class=" absolute left-0 top-0 w-full h-full" href=link_fn></a>
            <div class="relative z-[100] flex flex-col gap-6 shadow-lg bg-base00 rounded-lg px-6 py-4">
                { children() }
            </div>
        </div>
    }
}

// <p class="text-[1.5rem] text-base0A text-center">"Password Change"</p>
// <ErrGeneral id=move||"passowrd_change_general_error" error=general_errs/>
// {view_msg}
// <div class="ml-auto flex gap-2 ">
//     <Show when=when_confirm_btn>
//         <BtnPrimary id=move||"confirm_btn" is_loading on_click=on_confirm.clone()>{view_text}</BtnPrimary>
//     </Show>
//     <LinkSecondary id=move||"close_btn" link=link_back>"Cancel"</LinkSecondary>
// </div>
