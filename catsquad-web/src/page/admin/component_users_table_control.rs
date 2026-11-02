use super::UsersState;
use crate::{BtnPrimary, hook::Spawner};
use catsquad_client::XMLSender;
use catsquad_log::prelude::*;
use catsquad_shared::Order;
use leptos::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement};

#[component]
pub fn UserTableControls(spawner: Spawner, user_state: UsersState<XMLSender>) -> impl IntoView {
    let view_user_count = move || user_state.users.with(|v| v.len());

    let input_search_ord = NodeRef::new();
    let input_search_limit = NodeRef::new();
    let input_search_query = NodeRef::new();

    let search = move || {
        if let Some(input_query) = input_search_query
            .get_untracked()
            .map(|v: HtmlInputElement| v.value())
        {
            user_state.param_search_query.set(input_query);
        };

        spawner.spawn(async move {
            user_state.fetch().await;
        });
    };

    let input_search_ord_fn = move |e: web_sys::Event| {
        let Some(input_order) = input_search_ord
            .get_untracked()
            .and_then(|v: HtmlSelectElement| Order::try_from(v.value().as_str()).ok())
        else {
            return;
        };
        user_state.param_search_order.set(input_order);
    };

    let input_search_limit_fn = move |e: web_sys::Event| {
        let Some(input_limit) = input_search_limit
            .get_untracked()
            .and_then(|v: HtmlSelectElement| usize::from_str_radix(v.value().as_str(), 10).ok())
        else {
            return;
        };
        user_state.param_search_limit.set(input_limit);
    };

    let input_search_fn = move |e: web_sys::KeyboardEvent| {
        let key = e.key();
        if key.to_lowercase() != "enter" {
            return;
        }
        search();
    };

    let on_search_fn = move |_| {
        search();
    };

    view! {
        <div class="flex gap-2 place-items-center flex-wrap">
            <span>{view_user_count}" users"</span>
            <input
                autocomplete="off"
                id="input_search_query"
                name="input_search_query"
                placeholder="search"
                on:keydown=input_search_fn
                node_ref=input_search_query
                class="bg-base02 rounded-md px-2 py-1 "
                />
            <select node_ref=input_search_ord on:change=input_search_ord_fn class="bg-base02 rounded-md px-2 py-1" name="input_search_order" id="input_search_order">
                <option value="desc">desc</option>
                <option value="asc">asc</option>
            </select>
            <select node_ref=input_search_limit on:change=input_search_limit_fn class="bg-base02 rounded-md px-2 py-1" name="input_search_limit" id="input_search_limit">
                <option value="100">100</option>
                <option value="250">250</option>
                <option value="500">500</option>
                <option value="1000">1000</option>
                <option value="10000">10000</option>
            </select>
            <BtnPrimary on_click=on_search_fn>
                Search
            </BtnPrimary>
        </div>
    }
}
// <div class="flex flex-col gap-1">
//     <label for="input_search_limit">"limit"</label>
// </div>
