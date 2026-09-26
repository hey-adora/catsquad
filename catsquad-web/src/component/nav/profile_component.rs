use crate::{BtnPrimary, PageState, SVGGear, SVGProfile, SVGTriangle, hook::Spawner};
use catsquad_shared::{LINK_WEB_INDEX, LINK_WEB_SETTINGS};
use leptos::prelude::*;

#[component]
pub fn Profile(spawner: Spawner) -> impl IntoView {
    let page = PageState::get();

    let dropdown_open = RwSignal::new(false);
    let when_dropdown = move || dropdown_open.get();

    let on_open = move |_e| {
        dropdown_open.update(|v| *v = !*v);
    };

    let on_logout = move |_e| {
        spawner.spawn(async move {
            page.logout().await;
        });
    };

    let username = move || page.acc_username();

    view! {
        <div class="flex relative">
            <button id="profile_btn" on:click=on_open class="size-8 rounded-full bg-base03">
            </button>
            <Show when=when_dropdown >
                <SVGTriangle class="z-[100] size-4 text-base05 absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2"/>
                <div class="flex flex-col gap-2 text-left z-[100] px-4 py-2 rounded-md bg-base03 absolute right-0 bottom-0 transform translate-y-[calc(100%+0.5rem)] " >
                    <a class="flex gap-2 place-items-center px-1 " href=LINK_WEB_INDEX>
                        <SVGProfile class="size-6"/>
                        {username}
                    </a>
                    <a class="flex gap-2 place-items-center px-1 " href=LINK_WEB_SETTINGS>
                        <SVGGear class="size-6"/>
                        "Settings"
                    </a>
                    <BtnPrimary on_click=on_logout >
                        "logout"
                    </BtnPrimary>
                </div>
            </Show>

        </div>
    }
}
