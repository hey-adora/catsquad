use crate::{
    BtnDelete, BtnSize, ErrGeneral, Errs, ModalConfirm, Nav, hook::Spawner, page::create_client,
};
use leptos::prelude::*;

mod users_state;

use users_state::UsersState;

#[component]
pub fn Admin() -> impl IntoView {
    let spawner = Spawner::new();
    let user_state = UsersState::new(create_client());
    let user_errors = user_state.err_general;

    Effect::new(move || {
        spawner.spawn(async move {
            user_state.fetch().await;
        });
    });

    let view_users = move || {
        user_state
            .users
            .get()
            .into_iter()
            .map(|user| {
                let username = user.username;
                view! {
                      <tr class="bg-base01">
                        <td class="border">{username}</td>
                        <td class="border">
                            <BtnDelete size=BtnSize::Small>"delete"</BtnDelete>
                        </td>
                      </tr>
                }
            })
            .collect_view()
    };

    view! {
        <main class="">
            <Nav/>
            <ModalConfirm/>
            <Errs error=user_errors  />
            <table class="border-collapse border-spacing-[10rem] border">
              <thead>
                  <tr>
                    <th class="border">username</th>
                    <th class="border">controls</th>
                  </tr>
              </thead>
              <tbody>
                  {view_users}
              </tbody>
            </table>
        </main>
    }
}
