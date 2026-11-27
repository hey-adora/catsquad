use crate::{BtnPrimary, PageState, PfpAcc, SVGGear, SVGTriangle, hook::Spawner};
use catsquad_shared::{
    LINK_WEB_ADMIN, LINK_WEB_FOLLOWED, LINK_WEB_SETTINGS, RoleValue, link_relative_profile_gallery,
};
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
    let on_modal_open = move || {
        dropdown_open.set(false);
    };

    let username = move || page.acc_username();
    let link_profile = move || link_relative_profile_gallery(username());

    let is_logged_in = move || page.is_logged_in().unwrap_or_default();
    let is_perms_see_admin_controls =
        move || page.acc_contains_perm(RoleValue::PermissionSeeAdminControls);

    view! {
        <Show when=when_dropdown >
            <div on:click=on_open class="z-[99] w-[100dvw] h-[100dvh] bg-base01/30 fixed left-0 top-0">
            </div>
        </Show>
        <div class="flex relative">
            <button class="z-[100]" id="profile_btn" on:click=on_open >
                <PfpAcc class="size-8" no_link=true />
            </button>
            <Show when=when_dropdown >
                <SVGTriangle class="z-[100] size-4 text-base05 absolute left-[100%] top-[100%] transform -translate-x-1/2 -translate-y-1/2"/>
                <div class="flex flex-col gap-2 text-left z-[100] px-4 py-2 rounded-md bg-base03 absolute right-0 bottom-0 transform translate-y-[calc(100%+0.5rem)] " >
                    <a class="flex gap-2 place-items-center px-1 " href=link_profile>
                        <PfpAcc class="size-6"/>
                        {username}
                    </a>
                    <a class="flex gap-2 place-items-center px-1 " href=LINK_WEB_SETTINGS>
                        <SVGGear class="size-6"/>
                        "Settings"
                    </a>
                    <Show when=is_logged_in >
                        <a
                            class="pl-[2.2rem]"
                            href=LINK_WEB_FOLLOWED
                            >
                            "Followed"
                        </a>
                    </Show>
                    <Show when=is_perms_see_admin_controls>
                        <a
                            class="pl-[2.2rem]"
                            href=LINK_WEB_ADMIN
                            >
                            "Admin"
                        </a>
                    </Show>
                    <div class="text-center">
                        "---------"
                    </div>
                    <BtnPrimary
                        modal_enable=true
                        modal_title="logout"
                        modal_text="are you sure you want to logout?"
                        on_click=on_logout
                        on_modal_open=on_modal_open
                        >
                        "logout"
                    </BtnPrimary>
                </div>
            </Show>

        </div>
    }
}
