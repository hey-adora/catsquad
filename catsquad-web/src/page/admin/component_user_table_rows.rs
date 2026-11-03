use super::UsersState;
use crate::hook::Spawner;
use catsquad_client::XMLSender;
use catsquad_log::prelude::*;
use catsquad_shared::UserSearchRes;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

#[component]
pub fn UserTableRows(spawner: Spawner, user_state: UsersState<XMLSender>) -> impl IntoView {
    let view_users = move || {
        let user_len = user_state.users.with(|v| v.len());

        (0..user_len)
            .into_iter()
            .map(|i: usize| {
                view! {
                    <UserTableRow spawner user_state index=i/>
                }
            })
            .collect_view()
    };

    view! {
        {view_users}
    }
}

#[component]
pub fn UserTableRow(
    spawner: Spawner,
    user_state: UsersState<XMLSender>,
    index: usize,
    // user: UserSearchRes,
) -> impl IntoView {
    // TODO instead of bool, check array directly if the user is added
    let checked = RwSignal::new(false);
    let is_checked = move || checked.get() || user_state.check_all.get();

    let username = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.username.clone()))
            .unwrap_or_default()
    };
    let email = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.email.clone()))
            .unwrap_or_default()
    };
    let likes = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.likes.clone()))
            .unwrap_or_default()
    };
    let followers = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.followers.clone()))
            .unwrap_or_default()
    };
    let following = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.following.clone()))
            .unwrap_or_default()
    };
    let support = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.support.clone()))
            .unwrap_or_default()
    };
    let aboutme = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.aboutme.clone()))
            .unwrap_or_default()
    };
    let id = move || format!("{}_checkbox", username());

    let on_check_fn = move |_event| {
        checked.update(|v| *v = !*v);
        let is_checked = checked.get_untracked();

        let username = user_state
            .users
            .with_untracked(|v| v.get(index).map(|v| v.username.clone()))
            .unwrap_or_default();

        if is_checked {
            user_state.add_checked_username(username);
        } else {
            user_state.remove_checked_username(username);
        }
    };

    let row_class = move || {
        if is_checked() {
            Some(format!("bg-base06/20"))
        } else {
            None
        }
    };

    Effect::new(move || {
        let state = is_checked();
        let id = id();

        let Some(elm) = document().get_element_by_id(&id) else {
            return;
        };
        let elm: HtmlInputElement = elm.unchecked_into();

        trace!("checked {state}");

        elm.set_checked(state);
    });

    view! {
          <tr class=row_class>
            <td>
                <input on:change=on_check_fn type="checkbox" id=id.clone() name=id />
            </td>
            <td>{username}</td>
            <td>{email}</td>
            <td>{likes}</td>
            <td>{followers}</td>
            <td>{following}</td>
            <td>
                <textarea rows=1 cols=20>
                    {support}
                </textarea>
            </td>
            <td>
                <textarea rows=1 cols=20>
                    {aboutme}
                </textarea>
            </td>
          </tr>
    }
}
