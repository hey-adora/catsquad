use crate::{
    BtnDelete, BtnSize, ErrGeneral, Errs, ModalConfirm, Nav, hook::Spawner, page::create_client,
};
use catsquad_shared::UserSearchRes;
use leptos::prelude::*;

mod component_users_table_control;
mod users_state;

use component_users_table_control::UserTableControls;
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
            .map(|user: UserSearchRes| {
                let username = user.username;
                let email = user.email;
                let likes = user.likes;
                let followers = user.followers;
                let following = user.following;
                let support = user.support;
                let aboutme = user.aboutme;
                view! {
                      <tr class="bg-base01">
                        <td class="border">{username}</td>
                        <td class="border">{email}</td>
                        <td class="border">{likes}</td>
                        <td class="border">{followers}</td>
                        <td class="border">{following}</td>
                        <td class="border ">
                            <textarea rows=1 cols=20>
                                {support}
                            </textarea>
                        </td>
                        <td class="border ">
                            <textarea rows=1 cols=20>
                                {aboutme}
                            </textarea>
                        </td>
                        <td class="border">
                            <BtnDelete size=BtnSize::Small>"delete"</BtnDelete>
                        </td>
                      </tr>
                }
            })
            .collect_view()
    };

    let when_is_loading = move || spawner.is_busy.get();
    let when_is_not_loading = move || !when_is_loading();

    view! {
        <main class="">
            <Nav/>
            <ModalConfirm/>
            <div class="flex flex-col gap-2 place-items-start">
                <h2 class="font-bold text-[1.1rem] text-base06">"Users"</h2>
                <Errs error=user_errors  />
                <UserTableControls user_state spawner />

                <Show when=when_is_loading>
                    "loading..."
                </Show>

                <Show when=when_is_not_loading>
                    <table class="border-collapse border-spacing-[10rem] border">
                      <thead>
                          <tr>
                            <th class="border">"username"</th>
                            <th class="border">"email"</th>
                            <th class="border">"likes"</th>
                            <th class="border">"followers"</th>
                            <th class="border">"following"</th>
                            <th class="border">"support"</th>
                            <th class="border">"aboutme"</th>
                            <th class="border">"controls"</th>
                          </tr>
                      </thead>
                      <tbody>
                          {view_users}
                      </tbody>
                    </table>
                </Show>

            </div>
        </main>
    }
}
