use crate::{
    BtnDelete, BtnSize, ErrGeneral, Errs, ModalConfirm, Nav, hook::Spawner, page::create_client,
};
use catsquad_shared::UserSearchRes;
use leptos::prelude::*;

mod component_user_table_rows;
mod component_users_table_control;
mod users_state;

use component_user_table_rows::UserTableRows;
use component_users_table_control::UserTableControls;
use users_state::UsersState;

#[component]
pub fn Admin() -> impl IntoView {
    let spawner = Spawner::new();
    let user_state = UsersState::new(create_client());

    Effect::new(move || {
        spawner.spawn(async move {
            user_state.fetch().await;
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
                    <div class="mt-2 overflow-x-scroll max-w-full">
                        <table class="table-bordered border-collapse border-spacing-[10rem] border">
                          <thead>
                              <tr>
                                <th>
                                    <input on:change=on_check_all_fn type="checkbox" id="user_table_checkbox" name="user_table_checkbox" checked=user_state.check_all />
                                </th>
                                <th>"username"</th>
                                <th>"email"</th>
                                <th>"likes"</th>
                                <th>"followers"</th>
                                <th>"following"</th>
                                <th>"support"</th>
                                <th>"aboutme"</th>
                              </tr>
                          </thead>
                          <tbody>
                              <UserTableRows user_state spawner />
                          </tbody>
                        </table>
                    </div>
                </Show>

            </div>
        </main>
    }
}
