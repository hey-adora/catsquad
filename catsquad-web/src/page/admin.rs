use crate::{BtnSecondary, Errs, ModalConfirm, Nav, hook::Spawner, page::create_client};
use leptos::prelude::*;

mod component_user_table_rows;
mod component_users_table_control;
mod users_state;

use component_user_table_rows::UserTableRows;
use component_users_table_control::UserTableControls;
use users_state::UsersState;

#[component]
pub fn Admin() -> impl IntoView {
    // TODO show thier last comment and like count, comment count, etc

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

    let view_pages_text =
        move || format!("{}/{}", user_state.current_page(), user_state.max_page());

    let on_prev_page = move |_| {
        spawner.spawn(async move {
            user_state.prev_page();
            user_state.get_rows().await;
        })
    };

    let on_next_page = move |_| {
        spawner.spawn(async move {
            user_state.next_page();
            user_state.get_rows().await;
        })
    };

    let is_first_page = move || user_state.is_first_page();
    let is_last_page = move || user_state.is_last_page();

    // <td class="border">
    //     <BtnDelete size=BtnSize::Small>"delete"</BtnDelete>
    // </td>
    // <th class="border">"controls"</th>

    view! {
        <main class="">
            <Nav/>
            <ModalConfirm/>
            <div class="px-2 flex flex-col gap-2 place-items-start">
                <h2 class="font-bold text-[1.1rem] text-base06">"Users"</h2>

                <Errs error=user_state.err_fetch  />
                <Errs error=user_state.err_remove  />

                <UserTableControls user_state spawner />

                <Show when=when_is_loading>
                    "loading..."
                </Show>



                <Show when=when_is_not_loading>
                    <div class="grid grid-rows-[auto_auto] grid-cols-[1fr]">
                        <div class="flex gap-2 justify-end items-center">
                            <BtnSecondary disabled=is_first_page on_click=on_prev_page >"-"</BtnSecondary>
                            <span class="text-[1.1rem]">
                                {view_pages_text}
                            </span>
                            <BtnSecondary disabled=is_last_page on_click=on_next_page >"+"</BtnSecondary>
                        </div>
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
        </main>
    }
}
