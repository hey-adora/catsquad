use crate::{PageState, SVGTriangle, hook::Spawner};
use catsquad_shared::{LINK_WEB_INDEX, LINK_WEB_SETTINGS};
use leptos::prelude::*;
use web_sys::{MouseEvent, SubmitEvent};

#[component]
pub fn Profile(spawner: Spawner) -> impl IntoView {
    let page = PageState::get();

    let dropdown_open = RwSignal::new(false);
    let when_dropdown = move || dropdown_open.get();

    let on_open = move |_e| {
        dropdown_open.update(|v| *v = !*v);
    };

    let on_menu_click = move |e: MouseEvent| {
        // e.prevent_default();
        e.stop_propagation();
    };

    let on_logout = move |e: SubmitEvent| {
        e.prevent_default();

        spawner.spawn(async move {
            page.logout().await;
        });
    };

    let username = move || page.acc_username();

    view! {
        <button on:click=on_open class="size-8 rounded-full bg-base03 relative">
            <Show when=when_dropdown >
                <SVGTriangle class="z-[5] size-4 text-base05 absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2"/>
                <div on:click=on_menu_click class="text-left z-[1] px-4 py-2 rounded-md bg-base03 absolute left-[-100%] top-[calc(100%+2.5rem)] transform -translate-x-1/2 -translate-y-1/2 " >
                    <a href=LINK_WEB_INDEX>{username}</a>
                    <a href=LINK_WEB_SETTINGS>"Settings"</a>
                    <form method="POST" action="" on:submit=on_logout >
                        <input id="logout_btn" type="submit" value="logout" class=""/>
                    </form>
                </div>
            </Show>
        </button>
    }
}
// transition-all duration-300 ease-in hover:font-bold
