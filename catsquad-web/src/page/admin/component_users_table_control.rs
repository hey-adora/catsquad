use crate::hook::Spawner;

use super::UsersState;
use catsquad_client::XMLSender;
use catsquad_shared::{Order, UserSearchRes};
use leptos::prelude::*;
use web_sys::{HtmlSelectElement, MouseEvent};

#[component]
pub fn UserTableControls(
    #[prop(optional, into)] users: Signal<Vec<UserSearchRes>>,
    spawner: Spawner,
    user_state: UsersState<XMLSender>,
) -> impl IntoView {
    let view_user_count = move || users.with(|v| v.len());

    let input_ord = NodeRef::new();
    let on_change_fn = move |e: web_sys::Event| {
        let Some(input_order) = input_ord
            .get_untracked()
            .and_then(|v: HtmlSelectElement| Order::try_from(v.value().as_str()).ok())
        else {
            return;
        };
        user_state.param_order.set(input_order);
        spawner.spawn(async move {
            user_state.fetch().await;
        });
    };

    view! {
        <div class="flex gap-2">
            <span>{view_user_count}" users"</span>
            <div>"search"</div>
            <select node_ref=input_ord on:change=on_change_fn class="bg-base00" name="order" id="order">
                <option value="desc">desc</option>
                <option value="asc">asc</option>
            </select>
        </div>
    }
}
