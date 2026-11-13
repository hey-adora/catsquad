use super::UsersState;
use crate::hook::Spawner;
use catsquad_client::XMLSender;
use catsquad_log::prelude::*;
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
    // let checked = RwSignal::new(false);

    let username_tracked = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.username.clone()))
            .unwrap_or_default()
    };
    let username_untracked = move || {
        user_state
            .users
            .with_untracked(|v| v.get(index).map(|v| v.username.clone()))
            .unwrap_or_default()
    };
    let state = move || {
        user_state
            .users
            .with(|v| v.get(index).map(|v| v.state.clone()))
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

    let id = move || format!("{}_checkbox", username_tracked());
    let row_is_checked_tracked = move || {
        user_state.checked_usernames.with(|v| {
            let username = username_tracked();
            v.iter().any(|v| *v == username)
        })
    };
    let is_checked = move || row_is_checked_tracked() || user_state.check_all.get();

    let on_check_fn = move |_event| {
        let username = username_untracked();
        let is_checked = user_state
            .checked_usernames
            .with_untracked(|v| v.iter().any(|v| *v == username));

        trace!("on_check_fn {username} {is_checked}");

        if !is_checked {
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
            <td>{username_tracked}</td>
            <td>{state}</td>
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
