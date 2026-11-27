use crate::{
    Errs, ModalConfirm, Nav, PageState, hook::Spawner, page::create_client,
    page_state::CurrentRoute,
};
use catsquad_shared::{
    RoleValue, link_relative_admin, link_relative_admin_roles, link_relative_admin_users,
};
use catsquad_web_utils::turnary::Turnary;
use leptos::prelude::*;

mod component_user_table_rows;
mod component_users_table_control;
mod roles_state;
mod users_state;

use component_user_table_rows::UserTableRows;
use component_users_table_control::UserTableControls;
use leptos_router::hooks::use_location;
use users_state::UsersState;

#[component]
pub fn Admin() -> impl IntoView {
    // TODO show thier last comment and like count, comment count, etc
    let location = use_location();
    let page = PageState::get();

    let spawner = Spawner::new();
    let user_state = UsersState::new(create_client());

    Effect::new(move || {
        spawner.spawn(async move {
            user_state.init().await;
        });
    });

    let when_is_loading = move || spawner.is_busy.get();
    let when_is_not_loading = move || !when_is_loading();

    let on_check_all_fn = move |_| {
        user_state.check_all.update(|v| *v = !*v);
    };

    // <td class="border">
    //     <BtnDelete size=BtnSize::Small>"delete"</BtnDelete>
    // </td>
    // <th class="border">"controls"</th>

    let current_route = page.current_route(location);
    let when_tab_none = move || match current_route.get() {
        CurrentRoute::Admin => true,
        _ => false,
    };
    let when_tab_users = move || match current_route.get() {
        CurrentRoute::AdminUsers => true,
        _ => false,
    };
    let when_tab_roles = move || match current_route.get() {
        CurrentRoute::AdminRoles => true,
        _ => false,
    };

    let class_side = "rounded-lg self-center text-[1.2rem] px-4 py-2 font-bold ";
    let class_side_none = move || {
        format!(
            "{} {}",
            class_side,
            when_tab_none().either("bg-base05 text-base01", "bg-base02")
        )
    };
    let class_side_user = move || {
        format!(
            "{} {}",
            class_side,
            when_tab_users().either("bg-base05 text-base01", "bg-base02")
        )
    };
    let class_side_roles = move || {
        format!(
            "{} {}",
            class_side,
            when_tab_roles().either("bg-base05 text-base01", "bg-base02")
        )
    };
    let view_user_count = move || user_state.users.with(|v| v.len());

    let perm = move || RoleValue::PermissionRoleAdd.to_string();

    view! {
        <main class="">
            <Nav/>
            <ModalConfirm/>
            <div class="px-4 gap-4 grid grid-cols-[auto_1fr]">
                <div class="bg-base01 flex flex-col gap-4 px-2 py-3 bg-base01 rounded-lg">
                    <a href=link_relative_admin class=class_side_none >"*"</a>
                    <a href=link_relative_admin_users class=class_side_user >"U"</a>
                    <a href=link_relative_admin_roles class=class_side_roles >"R"</a>
                </div>

                <Show when=when_tab_none>
                    "admin"
                </Show>

                <Show when=when_tab_roles>
                    <div class="flex flex-col gap-2 ">
                        <h2 class="font-bold text-[1.1rem] text-base06">"Roles * "{0}</h2>

                        <Show when=when_is_loading>
                            "loading..."
                        </Show>

                        <Show when=when_is_not_loading>
                            <div class="grid grid-rows-[auto_auto] grid-cols-[1fr]">
                                <div class="mt-2 overflow-x-scroll max-w-full">
                                    <table class="table-bordered border-collapse border-spacing-[10rem] border">
                                      <thead>
                                          <tr>
                                            <th>
                                                <input on:change=on_check_all_fn type="checkbox" id="user_table_checkbox" name="user_table_checkbox" checked=user_state.check_all />
                                            </th>
                                            <th>"name"</th>
                                            <th>"permissions"</th>
                                          </tr>
                                      </thead>
                                      <tbody>
                                          <tr >
                                            <td>
                                                <input type="checkbox"/>
                                            </td>
                                            <td>"admin"</td>
                                            <td>{perm}</td>
                                          </tr>
                                      </tbody>
                                    </table>
                                </div>
                            </div>
                        </Show>
                    </div>

                </Show>

                <Show when=when_tab_users>
                    <div class="flex flex-col gap-2 ">

                        <h2 class="font-bold text-[1.1rem] text-base06">"Users * "{view_user_count}</h2>

                        <Errs error=user_state.err_fetch  />
                        <Errs error=user_state.err_remove  />

                        <UserTableControls user_state spawner />

                        <Show when=when_is_loading>
                            "loading..."
                        </Show>

                        <Show when=when_is_not_loading>
                            <div class="grid grid-rows-[auto_auto] grid-cols-[1fr]">
                                <div class="mt-2 overflow-x-scroll max-w-full">
                                    <table class="table-bordered border-collapse border-spacing-[10rem] border">
                                      <thead>
                                          <tr>
                                            <th>
                                                <input on:change=on_check_all_fn type="checkbox" id="user_table_checkbox" name="user_table_checkbox" checked=user_state.check_all />
                                            </th>
                                            <th>"ip"</th>
                                            <th>"username"</th>
                                            <th>"state"</th>
                                            <th>"email"</th>
                                            <th>"likes"</th>
                                            <th>"followers"</th>
                                            <th>"following"</th>
                                            <th>"agent"</th>
                                            <th>"support"</th>
                                            <th>"aboutme"</th>
                                          </tr>
                                      </thead>
                                      <tbody>
                                          <UserTableRows user_state spawner />
                                      </tbody>
                                    </table>
                                </div>
                            </div>
                        </Show>
                    </div>
                </Show>

            </div>
        </main>
    }
}
